import { api } from "../api";
import type { SavedConnection } from "../types";
import { EmptyState } from "./shared";

interface Props {
  connections: SavedConnection[];
  onRefresh: () => void;
}

export function VpnTab({ connections, onRefresh }: Props) {
  async function activate(uuid: string) {
    await api.activateConnection(uuid);
    onRefresh();
  }

  async function deactivate(uuid: string) {
    await api.deactivateConnection(uuid);
    onRefresh();
  }

  return (
    <div className="tab-stack">
      {connections.length === 0 ? (
        <EmptyState
          title="No VPN profiles"
          subtitle="Import VPN connections in NetworkManager first."
        />
      ) : (
        <div className="network-list">
          {connections.map((conn) => (
            <div key={conn.uuid} className="network-card">
              <div className="network-main">
                <div>
                  <strong>{conn.name}</strong>
                  <p>VPN profile</p>
                </div>
                <div className="network-actions">
                  {conn.active ? (
                    <>
                      <span className="pill active">Connected</span>
                      <button
                        className="ghost-btn small"
                        onClick={() => deactivate(conn.uuid)}
                      >
                        Disconnect
                      </button>
                    </>
                  ) : (
                    <button
                      className="primary-btn small"
                      onClick={() => activate(conn.uuid)}
                    >
                      Connect
                    </button>
                  )}
                </div>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
