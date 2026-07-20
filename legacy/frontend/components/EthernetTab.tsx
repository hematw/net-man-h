import { api } from "../api";
import type { SavedConnection } from "../types";
import { EmptyState } from "./shared";

interface Props {
  connections: SavedConnection[];
  onSelectConnection: (uuid: string) => void;
  onRefresh: () => void;
}

export function EthernetTab({
  connections,
  onSelectConnection,
  onRefresh,
}: Props) {
  const ethernet = connections.filter((c) => c.connectionType === "ethernet");

  async function activate(uuid: string) {
    await api.activateConnection(uuid);
    onRefresh();
  }

  return (
    <div className="tab-stack">
      {ethernet.length === 0 ? (
        <EmptyState
          title="No ethernet profiles"
          subtitle="Plug in a cable and NetworkManager will create one."
        />
      ) : (
        <div className="network-list">
          {ethernet.map((conn) => (
            <div key={conn.uuid} className="network-card">
              <div className="network-main">
                <div>
                  <strong>{conn.name}</strong>
                  <p>{conn.device || "No device"}</p>
                </div>
                <div className="network-actions">
                  {conn.active ? (
                    <span className="pill active">Active</span>
                  ) : (
                    <button
                      className="primary-btn small"
                      onClick={() => activate(conn.uuid)}
                    >
                      Connect
                    </button>
                  )}
                  <button
                    className="ghost-btn small"
                    onClick={() => onSelectConnection(conn.uuid)}
                  >
                    Configure
                  </button>
                </div>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
