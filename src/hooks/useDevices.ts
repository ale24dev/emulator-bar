import { useCallback, useEffect, useRef, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { api, errorMessage } from "../lib/api";
import type { Device, DeviceState } from "../lib/types";

const POLL_MS = 3000;
/** How long an optimistic transition is shown before trusting the backend again. */
const PENDING_TTL_MS = 90_000;

interface Pending {
  state: DeviceState;
  until: number;
}

const key = (d: Pick<Device, "platform" | "id">) => `${d.platform}:${d.id}`;

/**
 * Loads devices and polls while the popover is visible. Booting an Android
 * emulator takes a while to show up in adb, so transitions are tracked
 * optimistically until the backend reports a different state.
 */
export function useDevices(onError: (message: string) => void) {
  const [devices, setDevices] = useState<Device[]>([]);
  const [loading, setLoading] = useState(true);
  const [visible, setVisible] = useState(true);
  const pending = useRef(new Map<string, Pending>());
  const inFlight = useRef(false);
  const onErrorRef = useRef(onError);
  useEffect(() => {
    onErrorRef.current = onError;
  }, [onError]);

  const applyPending = useCallback((list: Device[]) => {
    const now = Date.now();
    return list.map((d) => {
      const p = pending.current.get(key(d));
      if (!p) return d;
      const settled =
        now > p.until ||
        (p.state === "booting" && d.state === "booted") ||
        (p.state === "shuttingDown" && d.state === "shutdown");
      if (settled) {
        pending.current.delete(key(d));
        return d;
      }
      return { ...d, state: p.state };
    });
  }, []);

  const refresh = useCallback(async () => {
    if (inFlight.current) return;
    inFlight.current = true;
    try {
      setDevices(applyPending(await api.listDevices()));
    } catch (err) {
      onErrorRef.current(errorMessage(err));
    } finally {
      inFlight.current = false;
      setLoading(false);
    }
  }, [applyPending]);

  const markPending = useCallback((d: Device, state: DeviceState) => {
    pending.current.set(key(d), { state, until: Date.now() + PENDING_TTL_MS });
    setDevices((list) => applyPending(list));
  }, [applyPending]);

  const clearPending = useCallback((d: Device) => {
    pending.current.delete(key(d));
  }, []);

  useEffect(() => {
    const win = getCurrentWindow();
    const unlisten = win.onFocusChanged(({ payload: focused }) => {
      setVisible(focused);
      if (focused) void refresh();
    });
    const onVisibility = () => setVisible(document.visibilityState === "visible");
    document.addEventListener("visibilitychange", onVisibility);
    return () => {
      void unlisten.then((fn) => fn());
      document.removeEventListener("visibilitychange", onVisibility);
    };
  }, [refresh]);

  useEffect(() => {
    if (!visible) return;
    void refresh();
    const timer = window.setInterval(() => void refresh(), POLL_MS);
    return () => window.clearInterval(timer);
  }, [visible, refresh]);

  return { devices, loading, refresh, markPending, clearPending };
}
