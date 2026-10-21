import type { Device } from "../lib/types";
import { DeviceRow, type DeviceActions } from "./DeviceRow";

interface DeviceGroupProps {
  title: string;
  devices: Device[];
  busyKeys: Set<string>;
  actions: DeviceActions;
  defaultUrl: string;
}

export function DeviceGroup({ title, devices, busyKeys, actions, defaultUrl }: DeviceGroupProps) {
  return (
    <section className="group">
      <h2 className="group-title">
        {title} <span className="muted">{devices.length}</span>
      </h2>
      {devices.length === 0 ? (
        <p className="empty">No devices found</p>
      ) : (
        <ul className="device-list">
          {devices.map((d) => (
            <DeviceRow
              key={d.id}
              device={d}
              busy={busyKeys.has(`${d.platform}:${d.id}`)}
              actions={actions}
              defaultUrl={defaultUrl}
            />
          ))}
        </ul>
      )}
    </section>
  );
}
