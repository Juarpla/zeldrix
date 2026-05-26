"use client";

interface ProgressBarProps {
  percentage: number;
  speedBps?: number;
  status: "downloading" | "verifying" | "completed" | "failed" | "cancelled";
}

export default function ProgressBar({ percentage, speedBps, status }: ProgressBarProps) {
  const formatSpeed = (bps: number) => {
    if (bps === 0) return "0 MB/s";
    const mbps = bps / (1024 * 1024);
    return `${mbps.toFixed(1)} MB/s`;
  };

  const getStatusColor = () => {
    switch (status) {
      case "downloading":
        return "#3b82f6";
      case "verifying":
        return "#eab308";
      case "completed":
        return "#22c55e";
      case "failed":
        return "#ef4444";
      case "cancelled":
        return "#6b7280";
      default:
        return "#6b7280";
    }
  };

  const getStatusText = () => {
    switch (status) {
      case "downloading":
        return speedBps ? `Descargando... ${formatSpeed(speedBps)}` : "Descargando...";
      case "verifying":
        return "Verificando hash SHA-256...";
      case "completed":
        return "Completado";
      case "failed":
        return "Error";
      case "cancelled":
        return "Cancelado";
      default:
        return "";
    }
  };

  return (
    <div style={{ width: "100%" }}>
      <div style={{ display: "flex", justifyContent: "space-between", fontSize: "12px", color: "#9ca3af", marginBottom: "4px" }}>
        <span>{getStatusText()}</span>
        <span>{percentage.toFixed(1)}%</span>
      </div>
      <div style={{ width: "100%", height: "8px", backgroundColor: "#374151", borderRadius: "9999px", overflow: "hidden" }}>
        <div
          style={{
            height: "100%",
            width: `${Math.min(percentage, 100)}%`,
            backgroundColor: getStatusColor(),
            transition: "width 300ms ease-out",
          }}
        />
      </div>
    </div>
  );
}