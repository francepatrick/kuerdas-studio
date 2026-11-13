export const LANE_HEIGHT = 106;
export const RULER_HEIGHT = 28;
export function draggedPosition(
  origin: number,
  deltaX: number,
  width: number,
  span: number,
  rate: number,
  snap: boolean,
) {
  const frame = origin + (deltaX / Math.max(1, width)) * span;
  const step = snap ? Math.max(1, Math.round(rate / 10)) : 1;
  return Math.max(0, Math.round(frame / step) * step);
}
export function droppedLane(clientY: number, top: number, count: number) {
  return Math.max(
    0,
    Math.min(
      count - 1,
      Math.floor((clientY - top - RULER_HEIGHT) / LANE_HEIGHT),
    ),
  );
}

export function overlapSegments(
  clips: { startFrame: number; frames: number }[],
) {
  const changes = new Map<number, number>();
  for (const clip of clips) {
    if (clip.frames <= 0) continue;
    changes.set(clip.startFrame, (changes.get(clip.startFrame) ?? 0) + 1);
    const end = clip.startFrame + clip.frames;
    changes.set(end, (changes.get(end) ?? 0) - 1);
  }
  let count = 0,
    last = 0;
  const segments: { start: number; end: number; count: number }[] = [];
  for (const [position, delta] of [...changes].sort((a, b) => a[0] - b[0])) {
    if (count > 1 && position > last)
      segments.push({ start: last, end: position, count });
    count += delta;
    last = position;
  }
  return segments;
}
