"use client";

import ProgressBar from "./ProgressBar";

export interface ModelInfo {
  id: string;
  name: string;
  model_type: string;
  vram: string;
  params: string;
  url: string;
  expected_hash: string;
  size_bytes: number;
}

export type ModelStatus = "not_downloaded" | "downloading" | "downloaded" | "loaded";

export interface ModelStatusInfo {
  info: ModelInfo;
  status: ModelStatus;
}

export interface DownloadProgress {
  model_id: string;
  bytes_downloaded: number;
  total_bytes: number | null;
  percentage: number;
  speed_bps: number;
  status: "pending" | "downloading" | "verifying" | "completed" | "failed" | "cancelled";
}

interface ModelCardProps {
  modelInfo: ModelStatusInfo;
  progress?: DownloadProgress;
  onDownload: (modelId: string) => void;
  onCancel: () => void;
  onUse: (modelId: string) => void;
}

export default function ModelCard({ modelInfo, progress, onDownload, onCancel, onUse }: ModelCardProps) {
  const { info, status } = modelInfo;

  const formatSize = (bytes: number) => {
    const gb = bytes / (1024 * 1024 * 1024);
    if (gb >= 1) {
      return `${gb.toFixed(1)} GB`;
    }
    const mb = bytes / (1024 * 1024);
    return `${mb.toFixed(0)} MB`;
  };

  const getTypeBadgeColor = (type: string) => {
    switch (type) {
      case "chat":
        return { backgroundColor: "rgba(168, 85, 247, 0.2)", color: "#c084fc" };
      case "embedding":
        return { backgroundColor: "rgba(34, 197, 94, 0.2)", color: "#4ade80" };
      case "specialized":
        return { backgroundColor: "rgba(249, 115, 22, 0.2)", color: "#fb923c" };
      default:
        return { backgroundColor: "rgba(107, 114, 128, 0.2)", color: "#9ca3af" };
    }
  };

  const getButtonStyle = (variant: "download" | "cancel" | "use" | "loaded") => {
    switch (variant) {
      case "download":
        return { backgroundColor: "rgba(59, 130, 246, 0.2)", color: "#60a5fa", hoverBg: "rgba(59, 130, 246, 0.3)" };
      case "cancel":
        return { backgroundColor: "rgba(239, 68, 68, 0.2)", color: "#f87171", hoverBg: "rgba(239, 68, 68, 0.3)" };
      case "use":
        return { backgroundColor: "rgba(34, 197, 94, 0.2)", color: "#4ade80", hoverBg: "rgba(34, 197, 94, 0.3)" };
      case "loaded":
        return { backgroundColor: "rgba(34, 197, 94, 0.2)", color: "#4ade80", hoverBg: "rgba(34, 197, 94, 0.3)" };
      default:
        return { backgroundColor: "rgba(107, 114, 128, 0.2)", color: "#9ca3af", hoverBg: "rgba(107, 114, 128, 0.3)" };
    }
  };

  const buttonStyle = getButtonStyle(
    status === "downloading" ? "cancel" :
    status === "downloaded" ? "use" :
    status === "loaded" ? "loaded" : "download"
  );

  const handleClick = () => {
    if (status === "downloading") {
      onCancel();
    } else if (status === "downloaded") {
      onUse(info.id);
    } else if (status === "not_downloaded") {
      onDownload(info.id);
    }
  };

  const getButtonText = () => {
    if (status === "downloading") return "Cancelar";
    if (status === "downloaded") return "Usar modelo";
    if (status === "loaded") return "En uso";
    return "Descargar";
  };

  const badgeStyle = getTypeBadgeColor(info.model_type);

  return (
    <div style={{
      backgroundColor: "rgba(31, 41, 55, 0.5)",
      border: "1px solid #374151",
      borderRadius: "12px",
      padding: "16px",
      display: "flex",
      flexDirection: "column",
      gap: "12px",
      transition: "border-color 200ms",
    }}>
      {/* Header */}
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start" }}>
        <div>
          <h3 style={{ fontWeight: "600", color: "#ffffff", fontSize: "14px" }}>{info.name}</h3>
          <p style={{ fontSize: "12px", color: "#9ca3af", marginTop: "2px" }}>{info.params} parámetros</p>
        </div>
        <span style={{
          padding: "4px 8px",
          borderRadius: "6px",
          fontSize: "12px",
          fontWeight: "500",
          ...badgeStyle,
        }}>
          {info.model_type}
        </span>
      </div>

      {/* Stats */}
      <div style={{ display: "flex", gap: "16px", fontSize: "12px", color: "#9ca3af" }}>
        <span>VRAM: {info.vram}</span>
        <span>Tamaño: {formatSize(info.size_bytes)}</span>
      </div>

      {/* Progress bar */}
      {status === "downloading" && progress && (
        <ProgressBar
          percentage={progress.percentage}
          speedBps={progress.speed_bps}
          status={progress.status === "verifying" ? "verifying" : "downloading"}
        />
      )}

      {/* Status message */}
      {status === "downloading" && progress?.status === "verifying" && (
        <div style={{ fontSize: "12px", color: "#eab308" }}>Verificando integridad del archivo...</div>
      )}

      {status === "downloading" && progress?.status === "failed" && (
        <div style={{ fontSize: "12px", color: "#ef4444" }}>Error en la descarga</div>
      )}

      {status === "not_downloaded" && (
        <div style={{ fontSize: "12px", color: "#6b7280" }}>No descargado</div>
      )}

      {status === "downloaded" && (
        <div style={{ fontSize: "12px", color: "#4ade80" }}>✓ Descargado y verificado</div>
      )}

      {/* Action */}
      <div style={{ marginTop: "auto", paddingTop: "8px" }}>
        <button
          onClick={handleClick}
          disabled={status === "loaded"}
          style={{
            padding: "8px 16px",
            borderRadius: "8px",
            fontSize: "14px",
            fontWeight: "500",
            backgroundColor: buttonStyle.backgroundColor,
            color: buttonStyle.color,
            border: "none",
            cursor: status === "loaded" ? "default" : "pointer",
            opacity: status === "loaded" ? 0.7 : 1,
            transition: "background-color 200ms",
          }}
        >
          {getButtonText()}
        </button>
      </div>
    </div>
  );
}