/** Formats a frame position as `mm:ss.mmm`. */
export function formatTime(frames: number, rate: number): string {
  const seconds = frames / Math.max(1, rate);
  const minutes = Math.floor(seconds / 60);
  const wholeSeconds = Math.floor(seconds % 60);
  const milliseconds = Math.floor((seconds % 1) * 1000);

  return `${pad(minutes, 2)}:${pad(wholeSeconds, 2)}.${pad(milliseconds, 3)}`;
}

function pad(value: number, length: number): string {
  return value.toString().padStart(length, "0");
}
