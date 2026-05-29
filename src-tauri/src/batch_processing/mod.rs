use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::RwLock;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::sidecar::{health, SidecarState};
use crate::structured_extraction::extract_structured_table_json_from_text;

const QUEUE_UPDATED_EVENT: &str = "batch-processing:queue-updated";
const ITEM_UPDATED_EVENT: &str = "batch-processing:item-updated";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BatchQueueStatus {
    Pending,
    Processing,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BatchDocumentInput {
    pub file_name: String,
    pub document_text: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct EnqueueBatchResult {
    pub batch_id: String,
    pub queued_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct BatchQueueItem {
    pub id: String,
    pub batch_id: String,
    pub order_index: usize,
    pub file_name: String,
    pub status: BatchQueueStatus,
    pub queued_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub result_json: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct BatchWorkItem {
    id: String,
    document_text: String,
}

pub struct BatchProcessingState {
    pub queue: RwLock<Vec<BatchQueueItem>>,
    tx: UnboundedSender<BatchWorkItem>,
    next_batch_number: AtomicUsize,
}

impl BatchProcessingState {
    pub fn new(tx: UnboundedSender<BatchWorkItem>) -> Self {
        Self {
            queue: RwLock::new(Vec::new()),
            tx,
            next_batch_number: AtomicUsize::new(1),
        }
    }

    fn next_batch_id(&self) -> String {
        let number = self.next_batch_number.fetch_add(1, Ordering::SeqCst);
        format!("batch-{number}")
    }
}

#[tauri::command]
pub fn enqueue_structured_extraction_batch(
    app_handle: AppHandle,
    state: State<'_, BatchProcessingState>,
    documents: Vec<BatchDocumentInput>,
) -> Result<EnqueueBatchResult, String> {
    if documents.is_empty() {
        return Err("Batch must include at least one document".to_string());
    }

    let batch_id = state.next_batch_id();
    let queued_count = documents.len();
    let (queue_items, work_items) = build_batch_items(&batch_id, documents);

    {
        let mut queue = state.queue.write().map_err(|error| error.to_string())?;
        queue.extend(queue_items);
    }

    for work_item in work_items {
        state
            .tx
            .send(work_item)
            .map_err(|error| format!("Failed to enqueue batch item: {error}"))?;
    }

    emit_queue_updated(&app_handle);

    Ok(EnqueueBatchResult {
        batch_id,
        queued_count,
    })
}

#[tauri::command]
pub fn get_structured_extraction_batch_queue(
    state: State<'_, BatchProcessingState>,
) -> Result<Vec<BatchQueueItem>, String> {
    let queue = state.queue.read().map_err(|error| error.to_string())?;
    Ok(queue.clone())
}

pub fn start_batch_processing_worker(
    app_handle: AppHandle,
    mut rx: UnboundedReceiver<BatchWorkItem>,
) {
    tauri::async_runtime::spawn(async move {
        while let Some(work_item) = rx.recv().await {
            mark_item_processing(&app_handle, &work_item.id);

            let result = process_batch_item(&app_handle, work_item.document_text).await;
            match result {
                Ok(result_json) => mark_item_completed(&app_handle, &work_item.id, result_json),
                Err(error) => mark_item_failed(&app_handle, &work_item.id, error),
            }
        }
    });
}

fn build_batch_items(
    batch_id: &str,
    documents: Vec<BatchDocumentInput>,
) -> (Vec<BatchQueueItem>, Vec<BatchWorkItem>) {
    documents
        .into_iter()
        .enumerate()
        .map(|(order_index, document)| {
            let id = format!("{batch_id}-{order_index}");
            let queued_at = Utc::now().to_rfc3339();
            let queue_item = BatchQueueItem {
                id: id.clone(),
                batch_id: batch_id.to_string(),
                order_index,
                file_name: document.file_name,
                status: BatchQueueStatus::Pending,
                queued_at,
                started_at: None,
                completed_at: None,
                result_json: None,
                error: None,
            };
            let work_item = BatchWorkItem {
                id,
                document_text: document.document_text,
            };

            (queue_item, work_item)
        })
        .unzip()
}

async fn process_batch_item(
    app_handle: &AppHandle,
    document_text: String,
) -> Result<String, String> {
    let sidecar_state = app_handle
        .try_state::<SidecarState>()
        .ok_or_else(|| "SidecarState is not managed".to_string())?;

    let port = {
        let guard = sidecar_state.0.lock().map_err(|error| error.to_string())?;
        guard.as_ref().map(|sidecar| sidecar.port)
    }
    .ok_or_else(|| "Sidecar not running. Start it first.".to_string())?;

    if !health::check_health(port).await {
        return Err(format!("Sidecar not responding on port {port}"));
    }

    extract_structured_table_json_from_text(document_text, port).await
}

fn mark_item_processing(app_handle: &AppHandle, item_id: &str) {
    update_item(app_handle, item_id, |item| {
        item.status = BatchQueueStatus::Processing;
        item.started_at = Some(Utc::now().to_rfc3339());
        item.error = None;
    });
}

fn mark_item_completed(app_handle: &AppHandle, item_id: &str, result_json: String) {
    update_item(app_handle, item_id, |item| {
        item.status = BatchQueueStatus::Completed;
        item.completed_at = Some(Utc::now().to_rfc3339());
        item.result_json = Some(result_json);
        item.error = None;
    });
}

fn mark_item_failed(app_handle: &AppHandle, item_id: &str, error: String) {
    update_item(app_handle, item_id, |item| {
        item.status = BatchQueueStatus::Failed;
        item.completed_at = Some(Utc::now().to_rfc3339());
        item.result_json = None;
        item.error = Some(error);
    });
}

fn update_item(
    app_handle: &AppHandle,
    item_id: &str,
    apply_update: impl FnOnce(&mut BatchQueueItem),
) {
    let Some(state) = app_handle.try_state::<BatchProcessingState>() else {
        return;
    };

    let updated_item = {
        let Ok(mut queue) = state.queue.write() else {
            return;
        };

        let Some(item) = queue.iter_mut().find(|item| item.id == item_id) else {
            return;
        };

        apply_update(item);
        item.clone()
    };

    let _ = app_handle.emit(ITEM_UPDATED_EVENT, &updated_item);
    emit_queue_updated(app_handle);
}

fn emit_queue_updated(app_handle: &AppHandle) {
    let _ = app_handle.emit(QUEUE_UPDATED_EVENT, ());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_batch_items_preserves_submission_order() {
        let documents = vec![
            BatchDocumentInput {
                file_name: "invoice-01.pdf".to_string(),
                document_text: "invoice one".to_string(),
            },
            BatchDocumentInput {
                file_name: "invoice-02.pdf".to_string(),
                document_text: "invoice two".to_string(),
            },
        ];

        let (queue_items, work_items) = build_batch_items("batch-test", documents);

        assert_eq!(queue_items[0].order_index, 0);
        assert_eq!(queue_items[1].order_index, 1);
        assert_eq!(work_items[0].id, "batch-test-0");
        assert_eq!(work_items[1].id, "batch-test-1");
    }

    #[test]
    fn failed_item_keeps_later_items_pending() {
        let documents = vec![
            BatchDocumentInput {
                file_name: "invoice-04.pdf".to_string(),
                document_text: "valid".to_string(),
            },
            BatchDocumentInput {
                file_name: "invoice-05.pdf".to_string(),
                document_text: "corrupt".to_string(),
            },
            BatchDocumentInput {
                file_name: "invoice-06.pdf".to_string(),
                document_text: "valid again".to_string(),
            },
        ];
        let (mut queue_items, _) = build_batch_items("batch-test", documents);

        queue_items[1].status = BatchQueueStatus::Failed;
        queue_items[1].error = Some("PDF is corrupt".to_string());

        assert!(matches!(queue_items[1].status, BatchQueueStatus::Failed));
        assert!(matches!(queue_items[2].status, BatchQueueStatus::Pending));
    }
}
