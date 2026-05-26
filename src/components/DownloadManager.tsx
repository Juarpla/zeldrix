"use client";

import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import ModelCard, { ModelStatusInfo, DownloadProgress } from "./ModelCard";

// Estilos inline para el contenedor principal
const containerStyle: React.CSSProperties = {
  minHeight: "100vh",
  backgroundColor: "#111827",
  color: "#ffffff",
};

const contentStyle: React.CSSProperties = {
  maxWidth: "1280px",
  margin: "0 auto",
  padding: "32px 16px",
};

const headerStyle: React.CSSProperties = {
  display: "flex",
  justifyContent: "space-between",
  alignItems: "center",
  marginBottom: "24px",
};

const titleStyle: React.CSSProperties = {
  fontSize: "24px",
  fontWeight: "700",
  color: "#ffffff",
  margin: 0,
};

const subtitleStyle: React.CSSProperties = {
  fontSize: "14px",
  color: "#9ca3af",
  marginTop: "4px",
};

const buttonStyle: React.CSSProperties = {
  padding: "6px 12px",
  fontSize: "14px",
  backgroundColor: "#374151",
  color: "#d1d5db",
  border: "none",
  borderRadius: "8px",
  cursor: "pointer",
};

const errorStyle: React.CSSProperties = {
  padding: "16px",
  backgroundColor: "rgba(239, 68, 68, 0.1)",
  border: "1px solid rgba(239, 68, 68, 0.2)",
  borderRadius: "8px",
  color: "#f87171",
  fontSize: "14px",
};

const gridStyle: React.CSSProperties = {
  display: "grid",
  gridTemplateColumns: "repeat(auto-fill, minmax(280px, 1fr))",
  gap: "16px",
};

const infoBoxStyle: React.CSSProperties = {
  marginTop: "32px",
  padding: "16px",
  backgroundColor: "rgba(31, 41, 55, 0.3)",
  border: "1px solid #374151",
  borderRadius: "8px",
};

const infoTitleStyle: React.CSSProperties = {
  fontSize: "14px",
  fontWeight: "500",
  color: "#ffffff",
  marginBottom: "8px",
};

const infoTextStyle: React.CSSProperties = {
  fontSize: "12px",
  color: "#9ca3af",
  lineHeight: "1.6",
};

const loadingStyle: React.CSSProperties = {
  display: "flex",
  justifyContent: "center",
  alignItems: "center",
  height: "256px",
  color: "#9ca3af",
};

export default function DownloadManager() {
  const [models, setModels] = useState<ModelStatusInfo[]>([]);
  const [downloadingModel, setDownloadingModel] = useState<string | null>(null);
  const [progress, setProgress] = useState<Record<string, DownloadProgress>>({});
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // Cargar lista de modelos
  const loadModels = useCallback(async () => {
    try {
      setLoading(true);
      const result = await invoke<ModelStatusInfo[]>("list_models");
      setModels(result);
      setError(null);
    } catch (e) {
      setError(`Error cargando modelos: ${e}`);
    } finally {
      setLoading(false);
    }
  }, []);

  // Escuchar eventos de descarga
  useEffect(() => {
    let unlistenProgress: UnlistenFn;
    let unlistenComplete: UnlistenFn;
    let unlistenError: UnlistenFn;
    let unlistenCancelled: UnlistenFn;
    let unlistenVerifying: UnlistenFn;

    const setupListeners = async () => {
      unlistenProgress = await listen<DownloadProgress>("download:progress", (event) => {
        const p = event.payload;
        setProgress((prev) => ({
          ...prev,
          [p.model_id]: p,
        }));
        setDownloadingModel(p.model_id);
      });

      unlistenVerifying = await listen<{ model_id: string }>("download:verifying", (event) => {
        const { model_id } = event.payload;
        setProgress((prev) => ({
          ...prev,
          [model_id]: {
            ...prev[model_id],
            status: "verifying",
          } as DownloadProgress,
        }));
      });

      unlistenComplete = await listen<{ model_id: string; path: string }>("download:complete", (event) => {
        const { model_id } = event.payload;
        setProgress((prev) => ({
          ...prev,
          [model_id]: {
            ...prev[model_id],
            status: "completed",
            percentage: 100,
          } as DownloadProgress,
        }));
        setDownloadingModel(null);
        loadModels();
      });

      unlistenError = await listen<{ model_id: string; error: string }>("download:error", (event) => {
        const { model_id, error } = event.payload;
        console.error(`Download error for ${model_id}:`, error);
        setProgress((prev) => ({
          ...prev,
          [model_id]: {
            ...prev[model_id],
            status: "failed",
          } as DownloadProgress,
        }));
        setDownloadingModel(null);
        loadModels();
      });

      unlistenCancelled = await listen<{ model_id: string }>("download:cancelled", (event) => {
        const { model_id } = event.payload;
        setProgress((prev) => ({
          ...prev,
          [model_id]: {
            ...prev[model_id],
            status: "cancelled",
          } as DownloadProgress,
        }));
        setDownloadingModel(null);
        loadModels();
      });
    };

    setupListeners();

    return () => {
      unlistenProgress?.();
      unlistenComplete?.();
      unlistenError?.();
      unlistenCancelled?.();
      unlistenVerifying?.();
    };
  }, [loadModels]);

  // Cargar modelos al montar
  useEffect(() => {
    loadModels();
  }, [loadModels]);

  // Polling para progreso (cada segundo mientras hay descarga activa)
  useEffect(() => {
    if (!downloadingModel) return;

    const interval = setInterval(async () => {
      try {
        const result = await invoke<DownloadProgress | null>("get_download_progress");
        if (result) {
          setProgress((prev) => ({
            ...prev,
            [result.model_id]: result,
          }));
          if (result.status === "completed" || result.status === "cancelled") {
            setDownloadingModel(null);
          }
        }
      } catch (e) {
        console.error("Error getting progress:", e);
      }
    }, 1000);

    return () => clearInterval(interval);
  }, [downloadingModel]);

  const handleDownload = async (modelId: string) => {
    try {
      setError(null);
      await invoke("download_model", { modelId });
      setDownloadingModel(modelId);
    } catch (e) {
      setError(`Error iniciando descarga: ${e}`);
    }
  };

  const handleCancel = async () => {
    try {
      await invoke("cancel_download");
      setDownloadingModel(null);
    } catch (e) {
      setError(`Error cancelando descarga: ${e}`);
    }
  };

  const handleUseModel = async (modelId: string) => {
    try {
      setError(null);
      const port = await invoke<number>("hot_swap_model", { modelId });
      console.log(`Hot swap successful, server running on port ${port}`);
      loadModels();
    } catch (e) {
      setError(`Error usando modelo: ${e}`);
    }
  };

  if (loading) {
    return (
      <div style={containerStyle}>
        <div style={contentStyle}>
          <div style={loadingStyle}>Cargando modelos...</div>
        </div>
      </div>
    );
  }

  return (
    <div style={containerStyle}>
      <div style={contentStyle}>
        {/* Header */}
        <div style={headerStyle}>
          <div>
            <h2 style={titleStyle}>Gestor de Modelos</h2>
            <p style={subtitleStyle}>
              Descarga y gestiona modelos de lenguaje desde Unsloth
            </p>
          </div>
          <button onClick={loadModels} style={buttonStyle}>
            Actualizar
          </button>
        </div>

        {/* Error message */}
        {error && <div style={errorStyle}>{error}</div>}

        {/* Models grid */}
        <div style={gridStyle}>
          {models.map((modelInfo) => (
            <ModelCard
              key={modelInfo.info.id}
              modelInfo={modelInfo}
              progress={progress[modelInfo.info.id]}
              onDownload={handleDownload}
              onCancel={handleCancel}
              onUse={handleUseModel}
            />
          ))}
        </div>

        {/* Info section */}
        <div style={infoBoxStyle}>
          <h3 style={infoTitleStyle}>Acerca de los modelos</h3>
          <p style={infoTextStyle}>
            Los modelos se descargan desde Hugging Face (Unsltoth) y se almacenan en el directorio de datos de la aplicación.
            Cada modelo es verificado con SHA-256 después de la descarga para garantizar su integridad.
            Puedes cambiar entre modelos sin reiniciar la aplicación usando la función &quot;Hot-Swap&quot;.
          </p>
        </div>
      </div>
    </div>
  );
}