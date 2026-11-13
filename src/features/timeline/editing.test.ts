import { expect, it } from 'vitest';
import { draggedPosition, droppedLane, overlapSegments } from './editing';
it('converts a drag to sample frames with optional tenth-second snap', () => {
  expect(draggedPosition(44100, 10, 1000, 441000, 44100, false)).toBe(48510);
  expect(draggedPosition(0, 13, 1000, 441000, 44100, true)).toBe(4410);
  expect(draggedPosition(0, -100, 1000, 441000, 44100, false)).toBe(0);
});
it('maps vertical drop positions to lanes, including a scrolled grid', () => {
  expect(droppedLane(250, 100, 4)).toBe(1);
  expect(droppedLane(50, -100, 4)).toBe(1);
  expect(droppedLane(-100, 100, 4)).toBe(0);
  expect(droppedLane(2000, 100, 4)).toBe(3);
});

it('shows shared intervals without treating touching clips as an overlap', () => {
  expect(
    overlapSegments([
      { startFrame: 0, frames: 10 },
      { startFrame: 10, frames: 5 },
      { startFrame: 2, frames: 0 },
    ]),
  ).toEqual([]);
  expect(
    overlapSegments([
      { startFrame: 0, frames: 10 },
      { startFrame: 5, frames: 10 },
      { startFrame: 7, frames: 1 },
    ]),
  ).toEqual([
    { start: 5, end: 7, count: 2 },
    { start: 7, end: 8, count: 3 },
    { start: 8, end: 10, count: 2 },
  ]);
});
