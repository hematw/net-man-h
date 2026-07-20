import { useEffect } from "react";
import { api } from "../api";
import type { ThemeColors } from "../types";

export function applyTheme(theme: ThemeColors) {
  const root = document.documentElement;
  root.style.setProperty("--accent", theme.accent);
  root.style.setProperty("--bg", theme.background);
  root.style.setProperty("--fg", theme.foreground);
  root.style.setProperty("--surface", theme.lighterBg);
  root.style.setProperty("--surface-2", theme.darkBg);
  root.style.setProperty("--muted", theme.muted);
  root.style.setProperty("--green", theme.green);
  root.style.setProperty("--red", theme.red);
  root.style.setProperty("--yellow", theme.yellow);
  root.style.setProperty("--blue", theme.blue);
  root.style.setProperty("--cyan", theme.cyan);
}

export function useTheme() {
  useEffect(() => {
    api.getTheme().then(applyTheme).catch(() => undefined);
  }, []);
}

export function signalBars(signal: number) {
  if (signal >= 80) return "▂▄▆█";
  if (signal >= 60) return "▂▄▆";
  if (signal >= 40) return "▂▄";
  if (signal >= 20) return "▂";
  return "·";
}

export function Spinner() {
  return <span className="spinner" aria-hidden="true" />;
}

export function ErrorBanner({ message }: { message: string }) {
  if (!message) return null;
  return <div className="error-banner">{message}</div>;
}

export function EmptyState({ title, subtitle }: { title: string; subtitle?: string }) {
  return (
    <div className="empty-state">
      <p className="empty-title">{title}</p>
      {subtitle ? <p className="empty-subtitle">{subtitle}</p> : null}
    </div>
  );
}
