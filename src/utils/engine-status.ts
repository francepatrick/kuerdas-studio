import type { EngineStatus } from "@/src/services/engine/engine.interfaces";

const DEVICE_STOPPED_MESSAGE =
  "The audio device stopped responding. Check the selected input/output devices, then restart audio.";

/**
 * The error to surface for a status poll. A device fault is only reported on the
 * poll where it first appears, so a dismissed message is not reasserted.
 */
export function statusError(
  status: EngineStatus,
  previousDeviceError: boolean,
): string | null {
  if (status.error) return status.error;

  return status.deviceError && !previousDeviceError
    ? DEVICE_STOPPED_MESSAGE
    : null;
}
