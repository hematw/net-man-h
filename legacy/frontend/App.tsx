import { useCallback, useEffect, useState } from "react";
import { api } from "./api";
import { ConnectionEditor } from "./components/ConnectionEditor";
import { EthernetTab } from "./components/EthernetTab";
import { HotspotTab } from "./components/HotspotTab";
import { OverviewTab } from "./components/OverviewTab";
import { Sidebar } from "./components/Sidebar";
import { StatusHeader } from "./components/StatusHeader";
import { useTheme } from "./components/shared";
import { VpnTab } from "./components/VpnTab";
import { WifiTab } from "./components/WifiTab";
import type {
  ConnectionDetails,
  HotspotStatus,
  NetworkDevice,
  NetworkStatus,
  SavedConnection,
  TabId,
  WifiNetwork,
} from "./types";
import "./App.css";

function App() {
  useTheme();

  const [tab, setTab] = useState<TabId>("overview");
  const [loading, setLoading] = useState(true);
  const [status, setStatus] = useState<NetworkStatus | null>(null);
  const [devices, setDevices] = useState<NetworkDevice[]>([]);
  const [connections, setConnections] = useState<SavedConnection[]>([]);
  const [vpnConnections, setVpnConnections] = useState<SavedConnection[]>([]);
  const [networks, setNetworks] = useState<WifiNetwork[]>([]);
  const [hotspot, setHotspot] = useState<HotspotStatus | null>(null);
  const [editor, setEditor] = useState<ConnectionDetails | null>(null);

  const refresh = useCallback(async (options?: { rescanWifi?: boolean }) => {
    setLoading(true);
    try {
      const [nextStatus, nextDevices, nextConnections, nextVpn, nextHotspot] =
        await Promise.all([
          api.getStatus(),
          api.getDevices(),
          api.getConnections(),
          api.getVpnConnections(),
          api.getHotspotStatus(),
        ]);

      setStatus(nextStatus);
      setDevices(nextDevices);
      setConnections(nextConnections);
      setVpnConnections(nextVpn);
      setHotspot(nextHotspot);

      if (nextStatus.wifiEnabled && (options?.rescanWifi || tab === "wifi")) {
        const nextNetworks = await api.scanWifi(Boolean(options?.rescanWifi));
        setNetworks(nextNetworks);
      } else if (!nextStatus.wifiEnabled) {
        setNetworks([]);
      }
    } finally {
      setLoading(false);
    }
  }, [tab]);

  useEffect(() => {
    refresh();
    const timer = window.setInterval(() => refresh(), 15000);
    return () => window.clearInterval(timer);
  }, [refresh]);

  useEffect(() => {
    if (tab === "wifi") {
      refresh({ rescanWifi: false });
    }
  }, [tab]);

  async function openEditor(uuid: string) {
    const details = await api.getConnectionDetails(uuid);
    setEditor(details);
  }

  return (
    <div className="app-shell">
      <Sidebar active={tab} onChange={setTab} />
      <main className="main-panel">
        <StatusHeader
          status={status}
          loading={loading}
          onRefresh={() => refresh({ rescanWifi: tab === "wifi" })}
        />

        {editor ? (
          <div className="editor-wrap">
            <button className="ghost-btn back-btn" onClick={() => setEditor(null)}>
              ← Back
            </button>
            <ConnectionEditor
              details={editor}
              onSaved={async () => {
                setEditor(null);
                await refresh();
              }}
            />
          </div>
        ) : (
          <>
            {tab === "overview" ? (
              <OverviewTab
                status={status}
                devices={devices}
                connections={connections}
                onSelectConnection={openEditor}
              />
            ) : null}
            {tab === "wifi" ? (
              <WifiTab
                networks={networks}
                wifiEnabled={status?.wifiEnabled ?? false}
                loading={loading}
                onRefresh={() => refresh({ rescanWifi: true })}
                onConnected={() => refresh({ rescanWifi: true })}
              />
            ) : null}
            {tab === "ethernet" ? (
              <EthernetTab
                connections={connections}
                onSelectConnection={openEditor}
                onRefresh={refresh}
              />
            ) : null}
            {tab === "vpn" ? (
              <VpnTab connections={vpnConnections} onRefresh={refresh} />
            ) : null}
            {tab === "hotspot" ? (
              <HotspotTab status={hotspot} onRefresh={refresh} />
            ) : null}
          </>
        )}
      </main>
    </div>
  );
}

export default App;
