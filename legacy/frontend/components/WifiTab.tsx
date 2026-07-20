import { useState } from "react";
import { api } from "../api";
import type { WifiNetwork } from "../types";
import { EmptyState, ErrorBanner, signalBars } from "./shared";

interface Props {
  networks: WifiNetwork[];
  wifiEnabled: boolean;
  loading: boolean;
  onRefresh: () => void;
  onConnected: () => void;
}

export function WifiTab({
  networks,
  wifiEnabled,
  loading,
  onRefresh,
  onConnected,
}: Props) {
  const [selected, setSelected] = useState<string | null>(null);
  const [password, setPassword] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");

  async function toggleWifi() {
    setBusy(true);
    setError("");
    try {
      await api.setWifiEnabled(!wifiEnabled);
      onRefresh();
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  }

  async function connect(ssid: string, secured: boolean, saved: boolean) {
    const needsPassword = secured && !saved;

    if (needsPassword && selected !== ssid) {
      setSelected(ssid);
      return;
    }

    setBusy(true);
    setError("");
    try {
      await api.connectWifi(ssid, needsPassword ? password : undefined);
      setSelected(null);
      setPassword("");
      onConnected();
    } catch (e) {
      setError(String(e));
      if (secured) {
        setSelected(ssid);
      }
    } finally {
      setBusy(false);
    }
  }

  async function disconnect() {
    setBusy(true);
    setError("");
    try {
      await api.disconnectWifi();
      onConnected();
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  }

  async function forget(ssid: string) {
    setBusy(true);
    setError("");
    try {
      await api.forgetWifi(ssid);
      onRefresh();
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="tab-stack">
      <div className="toolbar">
        <label className="toggle">
          <input
            type="checkbox"
            checked={wifiEnabled}
            onChange={toggleWifi}
            disabled={busy}
          />
          <span>WiFi {wifiEnabled ? "on" : "off"}</span>
        </label>
        <button className="ghost-btn" onClick={onRefresh} disabled={loading}>
          Scan
        </button>
        <button className="ghost-btn" onClick={disconnect} disabled={busy}>
          Disconnect
        </button>
      </div>

      <ErrorBanner message={error} />

      {!wifiEnabled ? (
        <EmptyState title="WiFi is turned off" subtitle="Enable WiFi to scan networks." />
      ) : networks.length === 0 ? (
        <EmptyState title="No networks found" subtitle="Try scanning again." />
      ) : (
        <div className="network-list">
          {networks.map((network) => {
            const secured = Boolean(network.security && network.security !== "--");
            const expanded = selected === network.ssid;

            return (
              <div key={`${network.ssid}-${network.bssid}`} className="network-card">
                <div className="network-main">
                  <div>
                    <strong>{network.ssid}</strong>
                    <p>
                      {secured ? network.security : "Open"} · {network.signal}%
                      {network.saved ? " · Saved" : ""}
                    </p>
                  </div>
                  <div className="network-actions">
                    <span className="signal">{signalBars(network.signal)}</span>
                    {network.inUse ? (
                      <span className="pill active">Connected</span>
                    ) : (
                      <button
                        className="primary-btn small"
                        onClick={() => connect(network.ssid, secured, network.saved)}
                        disabled={busy}
                      >
                        Connect
                      </button>
                    )}
                    <button
                      className="ghost-btn small"
                      onClick={() => forget(network.ssid)}
                      disabled={busy}
                    >
                      Forget
                    </button>
                  </div>
                </div>

                {expanded ? (
                  <div className="connect-form">
                    <input
                      type="password"
                      value={password}
                      onChange={(e) => setPassword(e.target.value)}
                      placeholder="Password"
                      autoFocus
                    />
                    <button
                      className="primary-btn small"
                      onClick={() => connect(network.ssid, true, false)}
                      disabled={busy || !password}
                    >
                      Join network
                    </button>
                  </div>
                ) : null}
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}
