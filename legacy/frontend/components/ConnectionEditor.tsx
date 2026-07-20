import { useState } from "react";
import { api } from "../api";
import type { ConnectionDetails, Ipv4Config, Ipv6Config } from "../types";

interface Props {
  details: ConnectionDetails;
  onSaved: () => void;
}

export function ConnectionEditor({ details, onSaved }: Props) {
  const [ipv4, setIpv4] = useState<Ipv4Config>({ ...details.ipv4 });
  const [ipv6, setIpv6] = useState<Ipv6Config>({ ...details.ipv6 });
  const [mtu, setMtu] = useState(details.mtu?.toString() ?? "");
  const [autoconnect, setAutoconnect] = useState(details.autoconnect);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState("");

  async function save() {
    setSaving(true);
    setError("");
    try {
      await api.saveConnection(
        details.uuid,
        ipv4,
        ipv6,
        mtu ? Number(mtu) : null,
        autoconnect,
      );
      await api.activateConnection(details.uuid);
      onSaved();
    } catch (e) {
      setError(String(e));
    } finally {
      setSaving(false);
    }
  }

  return (
    <div className="editor-card">
      <div className="editor-header">
        <div>
          <h3>{details.name}</h3>
          <p>{details.connectionType}</p>
        </div>
        <label className="toggle">
          <input
            type="checkbox"
            checked={autoconnect}
            onChange={(e) => setAutoconnect(e.target.checked)}
          />
          <span>Auto-connect</span>
        </label>
      </div>

      <section className="editor-section">
        <h4>IPv4</h4>
        <div className="segmented">
          <button
            className={ipv4.method !== "manual" ? "active" : ""}
            onClick={() => setIpv4({ ...ipv4, method: "auto" })}
          >
            DHCP
          </button>
          <button
            className={ipv4.method === "manual" ? "active" : ""}
            onClick={() => setIpv4({ ...ipv4, method: "manual" })}
          >
            Static
          </button>
        </div>
        {ipv4.method === "manual" ? (
          <div className="field-grid">
            <label>
              Address / prefix
              <input
                value={ipv4.addresses}
                onChange={(e) => setIpv4({ ...ipv4, addresses: e.target.value })}
                placeholder="192.168.1.50/24"
              />
            </label>
            <label>
              Gateway
              <input
                value={ipv4.gateway}
                onChange={(e) => setIpv4({ ...ipv4, gateway: e.target.value })}
                placeholder="192.168.1.1"
              />
            </label>
            <label className="full">
              DNS servers
              <input
                value={ipv4.dns}
                onChange={(e) => setIpv4({ ...ipv4, dns: e.target.value })}
                placeholder="1.1.1.1 8.8.8.8"
              />
            </label>
          </div>
        ) : null}
      </section>

      <section className="editor-section">
        <h4>IPv6</h4>
        <div className="segmented">
          <button
            className={ipv6.method === "auto" ? "active" : ""}
            onClick={() => setIpv6({ ...ipv6, method: "auto" })}
          >
            Auto
          </button>
          <button
            className={ipv6.method === "manual" ? "active" : ""}
            onClick={() => setIpv6({ ...ipv6, method: "manual" })}
          >
            Static
          </button>
          <button
            className={ipv6.method === "disabled" ? "active" : ""}
            onClick={() => setIpv6({ ...ipv6, method: "disabled" })}
          >
            Off
          </button>
        </div>
        {ipv6.method === "manual" ? (
          <div className="field-grid">
            <label>
              Address / prefix
              <input
                value={ipv6.addresses}
                onChange={(e) => setIpv6({ ...ipv6, addresses: e.target.value })}
                placeholder="fd00::10/64"
              />
            </label>
            <label>
              Gateway
              <input
                value={ipv6.gateway}
                onChange={(e) => setIpv6({ ...ipv6, gateway: e.target.value })}
                placeholder="fd00::1"
              />
            </label>
            <label className="full">
              DNS servers
              <input
                value={ipv6.dns}
                onChange={(e) => setIpv6({ ...ipv6, dns: e.target.value })}
                placeholder="2001:4860:4860::8888"
              />
            </label>
          </div>
        ) : null}
      </section>

      <section className="editor-section">
        <h4>Advanced</h4>
        <label>
          MTU
          <input
            value={mtu}
            onChange={(e) => setMtu(e.target.value)}
            placeholder="1500"
          />
        </label>
      </section>

      {error ? <p className="inline-error">{error}</p> : null}

      <div className="editor-actions">
        <button className="primary-btn" onClick={save} disabled={saving}>
          {saving ? "Saving…" : "Save & apply"}
        </button>
      </div>
    </div>
  );
}
