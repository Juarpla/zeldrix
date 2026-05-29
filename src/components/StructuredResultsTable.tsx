'use client';

import { useMemo, useState } from 'react';
import type {
  StructuredResultsTableJson,
  StructuredTableCell,
  StructuredTableColumn,
  StructuredTableDataType,
  StructuredTableRow,
} from '@/lib/types';
import { exportStructuredTableAsXlsx } from '@/lib/export-service';
import styles from './StructuredResultsTable.module.css';

interface StructuredResultsTableProps {
  table: StructuredResultsTableJson;
}

interface EditableRow {
  id: string;
  values: Record<string, string>;
}

interface CellValidation {
  status: 'valid' | 'warning' | 'error';
  message?: string;
}

const allowedDataTypes = new Set<StructuredTableDataType>([
  'string',
  'number',
  'integer',
  'boolean',
  'date',
  'currency',
]);

export function parseStructuredResultsTableJson(text: string): StructuredResultsTableJson | null {
  try {
    const parsed: unknown = JSON.parse(text);
    return isStructuredResultsTableJson(parsed) ? parsed : null;
  } catch {
    return null;
  }
}

function isStructuredResultsTableJson(value: unknown): value is StructuredResultsTableJson {
  if (!isRecord(value) || !Array.isArray(value.columns) || !Array.isArray(value.rows)) {
    return false;
  }

  return value.columns.every(isStructuredTableColumn) && value.rows.every(isStructuredTableRow);
}

function isStructuredTableColumn(value: unknown): value is StructuredTableColumn {
  return (
    isRecord(value) &&
    typeof value.name === 'string' &&
    allowedDataTypes.has(value.data_type as StructuredTableDataType) &&
    typeof value.nullable === 'boolean'
  );
}

function isStructuredTableRow(value: unknown): value is StructuredTableRow {
  return (
    isRecord(value) &&
    Array.isArray(value.cells) &&
    value.cells.every(isStructuredTableCell)
  );
}

function isStructuredTableCell(value: unknown): value is StructuredTableCell {
  return (
    isRecord(value) &&
    typeof value.column === 'string' &&
    typeof value.raw_value === 'string' &&
    typeof value.confidence === 'number' &&
    value.confidence >= 0 &&
    value.confidence <= 1 &&
    isStructuredCellValue(value.value)
  );
}

function isStructuredCellValue(value: unknown) {
  return (
    value === null ||
    typeof value === 'string' ||
    typeof value === 'number' ||
    typeof value === 'boolean'
  );
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null;
}

function createRows(table: StructuredResultsTableJson): EditableRow[] {
  return table.rows.map((row, rowIndex) => {
    const values = Object.fromEntries(
      table.columns.map((column) => {
        const cell = row.cells.find((candidate) => candidate.column === column.name);
        return [column.name, formatCellValue(cell?.value ?? null)];
      }),
    );

    return {
      id: `row-${rowIndex}`,
      values,
    };
  });
}

function formatCellValue(value: StructuredTableCell['value']) {
  if (value === null) {
    return '';
  }

  return String(value);
}

function validateCellValue(value: string, column: StructuredTableColumn): CellValidation {
  const trimmed = value.trim();

  if (!column.nullable && trimmed.length === 0) {
    return { status: 'error', message: 'Required value' };
  }

  if (value !== trimmed) {
    return { status: 'warning', message: 'Leading or trailing spaces' };
  }

  if (/\s{2,}/.test(value)) {
    return { status: 'warning', message: 'Repeated spacing' };
  }

  if (trimmed.length === 0) {
    return { status: 'valid' };
  }

  if ((column.data_type === 'number' || column.data_type === 'currency') && Number.isNaN(Number(trimmed))) {
    return { status: 'error', message: 'Use a numeric value' };
  }

  if (column.data_type === 'integer' && !/^-?\d+$/.test(trimmed)) {
    return { status: 'error', message: 'Use a whole number' };
  }

  if (column.data_type === 'boolean' && !/^(true|false|yes|no|si|sí)$/i.test(trimmed)) {
    return { status: 'error', message: 'Use true or false' };
  }

  if (column.data_type === 'date' && Number.isNaN(Date.parse(trimmed))) {
    return { status: 'error', message: 'Use a recognizable date' };
  }

  return { status: 'valid' };
}

export default function StructuredResultsTable({ table }: StructuredResultsTableProps) {
  const [columns, setColumns] = useState(() => table.columns);
  const [rows, setRows] = useState(() => createRows(table));
  const [draggedColumnName, setDraggedColumnName] = useState<string | null>(null);
  const [dragOverColumnName, setDragOverColumnName] = useState<string | null>(null);
  const [isExporting, setIsExporting] = useState(false);
  const [exportError, setExportError] = useState<string | null>(null);
  const [exportPath, setExportPath] = useState<string | null>(null);

  const totalWarnings = useMemo(() => {
    return rows.reduce((count, row) => {
      return count + columns.filter((column) => {
        const validation = validateCellValue(row.values[column.name] ?? '', column);
        return validation.status !== 'valid';
      }).length;
    }, 0);
  }, [columns, rows]);

  function updateCell(rowId: string, columnName: string, value: string) {
    setRows((currentRows) => currentRows.map((row) => {
      if (row.id !== rowId) {
        return row;
      }

      return {
        ...row,
        values: {
          ...row.values,
          [columnName]: value,
        },
      };
    }));
  }

  function deleteRow(rowId: string) {
    setRows((currentRows) => currentRows.filter((row) => row.id !== rowId));
  }

  function moveColumn(targetColumnName: string) {
    if (!draggedColumnName || draggedColumnName === targetColumnName) {
      return;
    }

    setColumns((currentColumns) => {
      const draggedIndex = currentColumns.findIndex((column) => column.name === draggedColumnName);
      const targetIndex = currentColumns.findIndex((column) => column.name === targetColumnName);

      if (draggedIndex < 0 || targetIndex < 0) {
        return currentColumns;
      }

      const nextColumns = [...currentColumns];
      const [draggedColumn] = nextColumns.splice(draggedIndex, 1);
      nextColumns.splice(targetIndex, 0, draggedColumn);
      return nextColumns;
    });
  }

  async function downloadExcel() {
    setIsExporting(true);
    setExportError(null);
    setExportPath(null);

    try {
      const result = await exportStructuredTableAsXlsx({
        columns,
        rows: rows.map((row) => row.values),
        filename: 'zeldrix-ai-table',
      });
      setExportPath(result.path);
    } catch (error) {
      setExportError(
        error instanceof Error
          ? error.message
          : typeof error === 'string'
            ? error
            : 'No se pudo exportar el Excel.',
      );
    } finally {
      setIsExporting(false);
    }
  }

  return (
    <section className={styles.shell} aria-label="Structured results table">
      <div className={styles.toolbar}>
        <div>
          <p className={styles.title}>Structured Results</p>
          <p className={styles.summary}>
            {rows.length} rows · {columns.length} columns · {totalWarnings} validations
          </p>
        </div>
        <button
          className={styles.exportButton}
          type="button"
          onClick={downloadExcel}
          disabled={isExporting || rows.length === 0}
        >
          {isExporting ? 'Exportando...' : 'Descargar Excel'}
        </button>
      </div>

      {(exportPath || exportError) && (
        <div className={exportError ? styles.exportError : styles.exportSuccess}>
          {exportError ?? `Excel guardado en ${exportPath}`}
        </div>
      )}

      {rows.length === 0 ? (
        <div className={styles.emptyState}>No rows remain in this result.</div>
      ) : (
        <div className={styles.scroller}>
          <table className={styles.table}>
            <thead>
              <tr>
                {columns.map((column) => (
                  <th
                    key={column.name}
                    className={`${styles.headerCell} ${dragOverColumnName === column.name ? styles.dragOver : ''}`}
                    scope="col"
                    onDragOver={(event) => {
                      event.preventDefault();
                      setDragOverColumnName(column.name);
                    }}
                    onDrop={(event) => {
                      event.preventDefault();
                      moveColumn(column.name);
                      setDraggedColumnName(null);
                      setDragOverColumnName(null);
                    }}
                  >
                    <button
                      className={styles.headerButton}
                      type="button"
                      draggable
                      title="Drag to reorder column"
                      onDragStart={() => setDraggedColumnName(column.name)}
                      onDragEnd={() => {
                        setDraggedColumnName(null);
                        setDragOverColumnName(null);
                      }}
                    >
                      <span className={styles.columnName}>{column.name}</span>
                      <span className={styles.columnMeta}>
                        {column.data_type}
                        {column.nullable ? ' · optional' : ' · required'}
                      </span>
                    </button>
                  </th>
                ))}
                <th className={styles.actionsHeader} scope="col">Rows</th>
              </tr>
            </thead>
            <tbody>
              {rows.map((row, rowIndex) => (
                <tr key={row.id}>
                  {columns.map((column) => {
                    const value = row.values[column.name] ?? '';
                    const validation = validateCellValue(value, column);

                    return (
                      <td className={styles.cell} key={`${row.id}-${column.name}`}>
                        <input
                          aria-label={`${column.name}, row ${rowIndex + 1}`}
                          className={`${styles.cellInput} ${validation.status === 'error' ? styles.cellInputInvalid : ''}`}
                          value={value}
                          onChange={(event) => updateCell(row.id, column.name, event.target.value)}
                        />
                        {validation.message && (
                          <div className={styles.validationMessage}>{validation.message}</div>
                        )}
                      </td>
                    );
                  })}
                  <td className={styles.rowActions}>
                    <button
                      className={styles.deleteButton}
                      type="button"
                      onClick={() => deleteRow(row.id)}
                    >
                      Delete
                    </button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </section>
  );
}
