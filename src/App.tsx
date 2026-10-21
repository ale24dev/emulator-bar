import { useCallback, useMemo, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { DeviceGroup } from "./components/DeviceGroup";
import type { DeviceActions } from "./components/DeviceRow";
import { ToastStack } from "./components/ToastStack";
import { Toolbar } from "./components/Toolbar";
import { useDevices } from "./hooks/useDevices";
import { useToasts } from "./hooks/useToasts";
import { api, errorMessage } from "./lib/api";
import type { BulkResult, Device } from "./lib/types";
import "./App.css";

const deviceKey = (d: Device) => `${d.platform}:${d.id}`;

function App() {
  const { toasts, push, dismiss } = useToasts();
  const reportError = useCallback((m: string) => push("error", m), [push]);
  const { devices, loading, refresh, markPending, clearPending } = useDevices(reportError);
  const [url, setUrl] = useState("");
  const [busyKeys, setBusyKeys] = useState<Set<string>>(new Set());
  const [bulkBusy, setBulkBusy] = useState(false);

  const ios = useMemo(() => devices.filter((d) => d.platform === "ios"), [devices]);
  const android = useMemo(() => devices.filter((d) => d.platform === "android"), [devices]);
  const bootedCount = devices.filter((d) => d.state === "booted").length;

  /** Runs one device action with busy state, toast feedback and a refresh. */
  const run = useCallback(
    async (d: Device, task: () => Promise<string | void>, success?: string) => {
      const k = deviceKey(d);
      setBusyKeys((s) => new Set(s).add(k));
      try {
        const result = await task();
        const message = typeof result === "string" ? result : success;
        if (message) push("success", message);
        return true;
      } catch (err) {
        push("error", errorMessage(err));
        return false;
      } finally {
        setBusyKeys((s) => {
          const next = new Set(s);
          next.delete(k);
          return next;
        });
        void refresh();
      }
    },
    [push, refresh],
  );

  const actions: DeviceActions = {
    toggle: (d) => {
      const booting = d.state !== "booted";
      markPending(d, booting ? "booting" : "shuttingDown");
      void run(d, () => (booting ? api.boot(d) : api.shutdown(d))).then((ok) => {
        if (!ok) clearPending(d);
      });
    },
    openUrl: (d, link) => void run(d, () => api.openUrl(d, link), `Opened on ${d.name}`),
    install: (d) =>
      void (async () => {
        const ext = d.platform === "ios" ? "app" : "apk";
        // Keep the popover open while the native file dialog has focus.
        await api.setHideOnBlur(false);
        let path: string | null = null;
        try {
          path = await open({
            multiple: false,
            directory: false,
            title: `Choose a .${ext} to install on ${d.name}`,
            filters: [{ name: ext === "app" ? "iOS app bundle" : "Android package", extensions: [ext] }],
          });
        } finally {
          await api.setHideOnBlur(true);
        }
        if (path) await run(d, () => api.install(d, path), `Installed on ${d.name}`);
      })().catch((err) => push("error", errorMessage(err))),
    screenshot: (d) =>
      void run(d, async () => `Saved ${(await api.screenshot(d)).split("/").pop()} to Desktop`),
    erase: (d) =>
      void run(
        d,
        () => api.erase(d),
        d.platform === "ios" ? `Erased ${d.name}` : `Wiping ${d.name}; it will cold boot`,
      ),
  };

  const runBulk = async (task: () => Promise<BulkResult>, verb: string) => {
    setBulkBusy(true);
    try {
      const { succeeded, failed } = await task();
      if (succeeded > 0) push("success", `${verb} ${succeeded} device${succeeded === 1 ? "" : "s"}`);
      if (failed.length > 0) push("error", failed.join("\n"));
    } catch (err) {
      push("error", errorMessage(err));
    } finally {
      setBulkBusy(false);
      void refresh();
    }
  };

  return (
    <main className="app">
      <header className="app-header">
        <h1>Emulators</h1>
      </header>
      <Toolbar
        url={url}
        onUrlChange={setUrl}
        bootedCount={bootedCount}
        busy={bulkBusy}
        onOpenAll={() => void runBulk(() => api.openUrlAllBooted(url), "Opened on")}
        onShutdownAll={() => void runBulk(api.shutdownAll, "Shut down")}
        onRefresh={() => void refresh()}
      />
      <div className="groups">
        {loading && devices.length === 0 ? (
          <p className="empty">Loading devices…</p>
        ) : (
          <>
            <DeviceGroup title="iOS" devices={ios} busyKeys={busyKeys} actions={actions} defaultUrl={url} />
            <DeviceGroup
              title="Android"
              devices={android}
              busyKeys={busyKeys}
              actions={actions}
              defaultUrl={url}
            />
          </>
        )}
      </div>
      <ToastStack toasts={toasts} onDismiss={dismiss} />
    </main>
  );
}

export default App;
