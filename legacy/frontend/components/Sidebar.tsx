import type { TabId } from "../types";

const tabs: { id: TabId; label: string; icon: string }[] = [
  { id: "overview", label: "Overview", icon: "◉" },
  { id: "wifi", label: "WiFi", icon: "⌁" },
  { id: "ethernet", label: "Ethernet", icon: "⎔" },
  { id: "vpn", label: "VPN", icon: "⛨" },
  { id: "hotspot", label: "Hotspot", icon: "◎" },
];

interface Props {
  active: TabId;
  onChange: (tab: TabId) => void;
}

export function Sidebar({ active, onChange }: Props) {
  return (
    <aside className="sidebar">
      <div className="brand">
        <div className="brand-mark">⌁</div>
        <div>
          <h1>Aether Net</h1>
          <p>NetworkManager</p>
        </div>
      </div>
      <nav className="nav">
        {tabs.map((tab) => (
          <button
            key={tab.id}
            className={`nav-item ${active === tab.id ? "active" : ""}`}
            onClick={() => onChange(tab.id)}
          >
            <span className="nav-icon">{tab.icon}</span>
            <span>{tab.label}</span>
          </button>
        ))}
      </nav>
    </aside>
  );
}
