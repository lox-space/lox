// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

interface PlaybackState {
  startMs: number;
  endMs: number;
  currentTime: number;
  playing: boolean;
  rate: number;
}

export const playback = $state<PlaybackState>({
  startMs: 0,
  endMs: 0,
  currentTime: 0,
  playing: false,
  rate: 60,
});

export function setBounds(startMs: number, endMs: number): void {
  playback.startMs = startMs;
  playback.endMs = endMs;
  if (playback.currentTime < startMs) playback.currentTime = startMs;
  if (playback.currentTime > endMs) playback.currentTime = endMs;
}

export function play(): void {
  if (playback.currentTime >= playback.endMs) {
    playback.currentTime = playback.startMs;
  }
  playback.playing = true;
}

export function pause(): void {
  playback.playing = false;
}

export function seek(t: number): void {
  if (t < playback.startMs) t = playback.startMs;
  if (t > playback.endMs) t = playback.endMs;
  playback.currentTime = t;
}

export function setRate(r: number): void {
  playback.rate = r;
}

/**
 * Advance currentTime by dt * rate (same unit as the bounds).
 *
 * In production +page.svelte sets bounds in milliseconds and calls tick
 * with `frame_dt_seconds * 1000` so playback runs at wall time × rate.
 * In tests, both bounds and tick arguments are dimensionless integers.
 */
export function tick(dt: number): void {
  if (!playback.playing) return;
  const next = playback.currentTime + dt * playback.rate;
  if (next >= playback.endMs) {
    playback.currentTime = playback.endMs;
    playback.playing = false;
  } else if (next < playback.startMs) {
    playback.currentTime = playback.startMs;
  } else {
    playback.currentTime = next;
  }
}
