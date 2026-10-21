import { useEffect, useRef, useState } from "react";
import type { Device } from "../lib/types";

export interface DeviceActions {
  toggle: (d: Device) => void;
  openUrl: (d: Device, url: string) => void;
  install: (d: Device) => void;
  screenshot: (d: Device) => void;
  erase: (d: Device) => void;
}

interface DeviceRowProps {
  device: Device;
  busy: boolean;
  actions: DeviceActions;
  defaultUrl: string;
}

type Mode = "idle" | "menu" | "url" | "confirmErase";

const STATE_LABEL: Record<Device["state"], string> = {
  booted: "Booted",
  booting: "Booting…",
  shuttingDown: "Shutting down…",
  shutdown: "Shut down",
  unknown: "Unknown",
};

export function DeviceRow({ device, busy, actions, defaultUrl }: DeviceRowProps) {
  const [mode, setMode] = useState<Mode>("idle");
  const [url, setUrl] = useState(defaultUrl);
  const rowRef = useRef<HTMLLIElement>(null);

  const booted = device.state === "booted";
  const transitioning = device.state === "booting" || device.state === "shuttingDown";

  useEffect(() => {
    if (mode !== "menu") return;
    const close = (e: MouseEvent) => {
      if (!rowRef.current?.contains(e.target as Node)) setMode("idle");
    };
    document.addEventListener("mousedown", close);
    return () => document.removeEventListener("mousedown", close);
  }, [mode]);

  const choose = (fn: () => void) => {
    setMode("idle");
    fn();
  };

  return (
    <li className="device" ref={rowRef}>
      <div className="device-main">
        <span className={`dot ${device.state}`} title={STATE_LABEL[device.state]} />
        <div className="device-text">
          <span className="device-name">{device.name}</span>
          <span className="device-meta">
            {[device.osVersion, STATE_LABEL[device.state]].filter(Boolean).join(" · ")}
          </span>
        </div>
        <button
          className={`btn small ${booted ? "" : "primary"}`}
          disabled={busy || transitioning}
          onClick={() => actions.toggle(device)}
        >
          {busy ? "…" : booted ? "Shutdown" : "Boot"}
        </button>
        <button
          className="btn small icon"
          aria-label="More actions"
          aria-expanded={mode === "menu"}
          onClick={() => setMode(mode === "menu" ? "idle" : "menu")}
        >
          ⋯
        </button>
      </div>

      {mode === "menu" && (
        <div className="menu" role="menu">
          <button
            role="menuitem"
            disabled={!booted}
            onClick={() => {
              if (defaultUrl.trim()) setUrl(defaultUrl);
              setMode("url");
            }}
          >
            Open URL…
          </button>
          <button role="menuitem" disabled={!booted} onClick={() => choose(() => actions.install(device))}>
            Install {device.platform === "ios" ? ".app" : ".apk"}…
          </button>
          <button role="menuitem" disabled={!booted} onClick={() => choose(() => actions.screenshot(device))}>
            Screenshot to Desktop
          </button>
          <div className="menu-sep" />
          <button
            role="menuitem"
            className="danger"
            disabled={device.state !== "shutdown"}
            title={device.state !== "shutdown" ? "Shut down the device first" : undefined}
            onClick={() => setMode("confirmErase")}
          >
            {device.platform === "ios" ? "Erase…" : "Wipe data…"}
          </button>
        </div>
      )}

      {mode === "url" && (
        <form
          className="inline-panel"
          onSubmit={(e) => {
            e.preventDefault();
            choose(() => actions.openUrl(device, url));
          }}
        >
          <input
            className="input"
            autoFocus
            spellCheck={false}
            placeholder="https://… or myapp://…"
            value={url}
            onChange={(e) => setUrl(e.target.value)}
            onKeyDown={(e) => e.key === "Escape" && setMode("idle")}
          />
          <button className="btn small primary" type="submit" disabled={!url.trim()}>
            Open
          </button>
          <button className="btn small" type="button" onClick={() => setMode("idle")}>
            Cancel
          </button>
        </form>
      )}

      {mode === "confirmErase" && (
        <div className="inline-panel confirm">
          <span>
            {device.platform === "ios"
              ? "Erase all content and settings?"
              : "Wipe all data? The emulator will cold boot."}
          </span>
          <button className="btn small danger" onClick={() => choose(() => actions.erase(device))}>
            {device.platform === "ios" ? "Erase" : "Wipe"}
          </button>
          <button className="btn small" onClick={() => setMode("idle")}>
            Cancel
          </button>
        </div>
      )}
    </li>
  );
}
