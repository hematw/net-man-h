import type { NetworkStatus } from "../types";

interface Props {
  status: NetworkStatus | null;
  loading: boolean;
  onRefresh: () => void;
}

export function StatusHeader({ status, loading, onRefresh }: Props) {
  return (
    <header className="status-header">
      <div>
        <p className="eyebrow">Current connection</p>
        <h2>{status?.connected ? status.ssid || "Connected" : "Not connected"}</h2>
        <p className="status-meta">
          {status?.connected
            ? `${status.ip4 || "No IP"} · ${status.gateway || "No gateway"} · ${status.device}`
            : status?.wifiEnabled
              ? "WiFi radio on"
              : "WiFi radio off"}
        </p>
      </div>
      <button className="ghost-btn" onClick={onRefresh} disabled={loading}>
        {loading ? "Refreshing…" : "Refresh"}
      </button>
    </header>
  );
}
