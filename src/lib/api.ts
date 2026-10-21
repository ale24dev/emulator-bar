import { invoke } from "@tauri-apps/api/core";
import type { BulkResult, Device } from "./types";

type Target = Pick<Device, "platform" | "id">;

const target = (d: Target) => ({ platform: d.platform, id: d.id });

/** Typed wrappers around the Rust commands. Errors reject with a string. */
export const api = {
  listDevices: () => invoke<Device[]>("list_devices"),
  boot: (d: Target) => invoke<void>("boot_device", target(d)),
  shutdown: (d: Target) => invoke<void>("shutdown_device", target(d)),
  openUrl: (d: Target, url: string) => invoke<void>("open_url", { ...target(d), url }),
  install: (d: Target, path: string) => invoke<void>("install_app", { ...target(d), path }),
  screenshot: (d: Device) => invoke<string>("take_screenshot", { ...target(d), name: d.name }),
  erase: (d: Target) => invoke<void>("erase_device", target(d)),
  shutdownAll: () => invoke<BulkResult>("shutdown_all"),
  openUrlAllBooted: (url: string) => invoke<BulkResult>("open_url_all_booted", { url }),
  setHideOnBlur: (enabled: boolean) => invoke<void>("set_hide_on_blur", { enabled }),
};

export function errorMessage(err: unknown): string {
  if (typeof err === "string") return err;
  if (err instanceof Error) return err.message;
  return String(err);
}
