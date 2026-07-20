import type { NetworkDevice, NetworkStatus, SavedConnection } from "../types";
import { EmptyState } from "./shared";

interface Props {
  status: NetworkStatus | null;
  devices: NetworkDevice[];
  connections: SavedConnection[];
  onSelectConnection: (uuid: string) => void;
}

export function OverviewTab({
  status,
  devices,
  connections,
  onSelectConnection,
}: Props) {
  const activeConnections = connections.filter((c) => c.active);

  return (
    <div className="tab-grid">
      <section className="panel">
        <h3>Status</h3>
        <div className="stat-grid">
          <div className="stat-card">
            <span>Connection</span>
            <strong>{status?.connected ? "Online" : "Offline"}</strong>
          </div>
          <div className="stat-card">
            <span>Interface</span>
            <strong>{status?.device || "—"}</strong>
          </div>
          <div className="stat-card">
            <span>IPv4</span>
            <strong>{status?.ip4 || "—"}</strong>
          </div>
          <div className="stat-card">
            <span>Gateway</span>
            <strong>{status?.gateway || "—"}</strong>
          </div>
        </div>
      </section>

      <section className="panel">
        <h3>Devices</h3>
        {devices.length === 0 ? (
          <EmptyState title="No devices found" />
        ) : (
          <div className="list">
            {devices.map((device) => (
              <div key={device.name} className="list-row">
                <div>
                  <strong>{device.name}</strong>
                  <p>{device.deviceType} · {device.state}</p>
                </div>
                <span>{device.ip4 || device.connection || "—"}</span>
              </div>
            ))}
          </div>
        )}
      </section>

      <section className="panel wide">
        <h3>Active profiles</h3>
        {activeConnections.length === 0 ? (
          <EmptyState title="No active connections" />
        ) : (
          <div className="list">
            {activeConnections.map((conn) => (
              <button
                key={conn.uuid}
                className="list-row clickable"
                onClick={() => onSelectConnection(conn.uuid)}
              >
                <div>
                  <strong>{conn.name}</strong>
                  <p>{conn.connectionType} · {conn.device}</p>
                </div>
                <span className="pill active">Active</span>
              </button>
            ))}
          </div>
        )}
      </section>
    </div>
  );
}
