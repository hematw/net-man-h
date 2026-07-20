import { useState } from "react";
import { api } from "../api";
import type { HotspotStatus } from "../types";
import { ErrorBanner } from "./shared";

interface Props {
  status: HotspotStatus | null;
  onRefresh: () => void;
}

export function HotspotTab({ status, onRefresh }: Props) {
  const [ssid, setSsid] = useState("Aether Hotspot");
  const [password, setPassword] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");

  async function start() {
    setBusy(true);
    setError("");
    try {
      await api.createHotspot(ssid, password);
      onRefresh();
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  }

  async function stop() {
    setBusy(true);
    setError("");
    try {
      await api.stopHotspot();
      onRefresh();
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="tab-stack">
      <section className="panel">
        <h3>WiFi hotspot</h3>
        <p className="panel-copy">
          Share your connection over WiFi. Uses NetworkManager hotspot mode.
        </p>

        {status?.active ? (
          <div className="hotspot-active">
            <div>
              <strong>{status.ssid}</strong>
              <p>Broadcasting on {status.device}</p>
            </div>
            <button className="ghost-btn" onClick={stop} disabled={busy}>
              Stop hotspot
            </button>
          </div>
        ) : (
          <div className="field-grid">
            <label>
              Network name
              <input value={ssid} onChange={(e) => setSsid(e.target.value)} />
            </label>
            <label>
              Password
              <input
                type="password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                placeholder="At least 8 characters"
              />
            </label>
            <div className="editor-actions">
              <button
                className="primary-btn"
                onClick={start}
                disabled={busy || !ssid || password.length < 8}
              >
                Start hotspot
              </button>
            </div>
          </div>
        )}
      </section>

      <ErrorBanner message={error} />
    </div>
  );
}
