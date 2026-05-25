// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

export type Status = "idle" | "computing" | "done" | "error" | "cancelled";

interface StatusState {
  status: Status;
  pairsReceived: number;
  pairsExpected: number;
  lastDurationMs: number | null;
  lastError: string | null;
}

export const status = $state<StatusState>({
  status: "idle",
  pairsReceived: 0,
  pairsExpected: 0,
  lastDurationMs: null,
  lastError: null,
});

export function markStart(pairsExpected: number): void {
  status.status = "computing";
  status.pairsReceived = 0;
  status.pairsExpected = pairsExpected;
  status.lastError = null;
}

export function bumpPair(): void {
  status.pairsReceived += 1;
}

export function markDone(elapsedMs: number): void {
  status.status = "done";
  status.lastDurationMs = elapsedMs;
}

export function markCancelled(): void {
  status.status = "cancelled";
}

export function markError(message: string): void {
  status.status = "error";
  status.lastError = message;
}
