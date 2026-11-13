import { LANE_HEIGHT, RULER_HEIGHT } from "@/src/constants/Timeline";

export interface FrameSpan {
  frames: number;
  startFrame: number;
}

export interface OverlapSegment {
  count: number;
  end: number;
  start: number;
}

/** Frame a clip lands on after a horizontal drag, snapped to 100 ms when `snap` is set. */
export function draggedPosition(
  origin: number,
  deltaX: number,
  width: number,
  span: number,
  rate: number,
  snap: boolean,
): number {
  const frame = origin + (deltaX / Math.max(1, width)) * span;
  const step = snap ? Math.max(1, Math.round(rate / 10)) : 1;
  return Math.max(0, Math.round(frame / step) * step);
}
/** Lane index under a pointer, clamped to the existing lanes. */
export function droppedLane(
  clientY: number,
  top: number,
  count: number,
): number {
  return Math.max(
    0,
    Math.min(
      count - 1,
      Math.floor((clientY - top - RULER_HEIGHT) / LANE_HEIGHT),
    ),
  );
}

/** Ranges where more than one clip overlaps, with how many clips stack there. */
export function overlapSegments(clips: readonly FrameSpan[]): OverlapSegment[] {
  const changes = new Map<number, number>();
  for (const clip of clips) {
    if (clip.frames <= 0) continue;
    changes.set(clip.startFrame, (changes.get(clip.startFrame) ?? 0) + 1);
    const end = clip.startFrame + clip.frames;
    changes.set(end, (changes.get(end) ?? 0) - 1);
  }
  let count = 0;
  let last = 0;
  const segments: OverlapSegment[] = [];
  for (const [position, delta] of [...changes].sort((a, b) => a[0] - b[0])) {
    if (count > 1 && position > last)
      segments.push({ start: last, end: position, count });
    count += delta;
    last = position;
  }
  return segments;
}
