// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

import { describe, it, expect, beforeEach } from "vitest";
import {
  playback, setBounds, tick, play, pause, seek, setRate,
} from "./playback.svelte";

beforeEach(() => {
  setBounds(0, 1000);
  pause();
  seek(0);
  setRate(1);
});

describe("playback", () => {
  it("starts paused at the lower bound", () => {
    setBounds(100, 200);
    seek(100);
    expect(playback.playing).toBe(false);
    expect(playback.currentTime).toBe(100);
  });

  it("tick() advances currentTime by dt * rate while playing", () => {
    play();
    setRate(10);
    tick(0.5); // 0.5 * 10 = 5 simulated units
    expect(playback.currentTime).toBe(5);
  });

  it("tick() is a no-op when paused", () => {
    pause();
    tick(0.5);
    expect(playback.currentTime).toBe(0);
  });

  it("clamps currentTime to bounds and auto-pauses at end", () => {
    setBounds(0, 100);
    play();
    setRate(1);
    tick(150);
    expect(playback.currentTime).toBe(100);
    expect(playback.playing).toBe(false);
  });

  it("seek() clamps to bounds", () => {
    setBounds(0, 100);
    seek(-10);
    expect(playback.currentTime).toBe(0);
    seek(500);
    expect(playback.currentTime).toBe(100);
  });
});
