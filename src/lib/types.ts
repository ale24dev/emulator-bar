export type Platform = "android" | "ios";

export type DeviceState = "booted" | "booting" | "shuttingDown" | "shutdown" | "unknown";

export interface Device {
  id: string;
  name: string;
  platform: Platform;
  state: DeviceState;
  osVersion: string | null;
  serial: string | null;
}

export interface BulkResult {
  succeeded: number;
  failed: string[];
}
