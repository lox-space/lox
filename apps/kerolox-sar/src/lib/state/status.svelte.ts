// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

export type Status = "idle" | "computing" | "done" | "error" | "cancelled";

export interface StatusState {
  status: Status;
  received: number;
  expected: number;
  lastDurationMs: number | null;
  lastError: string | null;
}

export interface StatusController {
  state: StatusState;
  markStart(expected: number): void;
  bump(): void;
  markDone(elapsedMs: number): void;
  markCancelled(): void;
  markError(message: string): void;
}

function makeController(state: StatusState): StatusController {
  return {
    state,
    markStart(expected: number): void {
      state.status = "computing";
      state.received = 0;
      state.expected = expected;
      state.lastError = null;
    },
    bump(): void {
      state.received += 1;
    },
    markDone(elapsedMs: number): void {
      state.status = "done";
      state.lastDurationMs = elapsedMs;
    },
    markCancelled(): void {
      state.status = "cancelled";
    },
    markError(message: string): void {
      state.status = "error";
      state.lastError = message;
    },
  };
}

/** Status of the streaming `ComputeAccess` RPC. */
const accessState = $state<StatusState>({
  status: "idle",
  received: 0,
  expected: 0,
  lastDurationMs: null,
  lastError: null,
});
export const accessStatus = makeController(accessState);

/** Status of the streaming `PropagateTrajectories` RPC. */
const propagationState = $state<StatusState>({
  status: "idle",
  received: 0,
  expected: 0,
  lastDurationMs: null,
  lastError: null,
});
export const propagationStatus = makeController(propagationState);
