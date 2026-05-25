// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

import { createClient } from "@connectrpc/connect";
import { createConnectTransport } from "@connectrpc/connect-web";
import {
  Kerolox,
  type AccessRequest,
  type AccessPairResult,
  type PropagateRequest,
  type SampledTrajectoryMessage,
} from "@kerolox/proto-ts";

const ENGINE_URL = (import.meta.env?.VITE_ENGINE_URL as string | undefined) ?? "http://127.0.0.1:8080";

const transport = createConnectTransport({ baseUrl: ENGINE_URL, useBinaryFormat: false });
const client = createClient(Kerolox, transport);

export interface StreamCallbacks {
  onStart: () => void;
  onPair: (p: AccessPairResult) => void;
  onDone: (elapsedMs: number) => void;
  onCancel: () => void;
  onError: (err: Error) => void;
}

/**
 * Run a server-streaming ComputeAccess call. Honours `signal` for
 * cancellation; reports lifecycle events through `callbacks`.
 *
 * Returns when the stream ends, errors, or is aborted.
 */
export async function runComputeAccess(
  req: AccessRequest,
  cb: StreamCallbacks,
  signal: AbortSignal,
): Promise<void> {
  if (signal.aborted) {
    cb.onCancel();
    return;
  }
  cb.onStart();
  const startedAt = performance.now();
  try {
    const stream = client.computeAccess(req, { signal });
    for await (const pair of stream) {
      cb.onPair(pair);
    }
    cb.onDone(performance.now() - startedAt);
  } catch (err) {
    if (signal.aborted) {
      cb.onCancel();
    } else {
      cb.onError(err instanceof Error ? err : new Error(String(err)));
    }
  }
}

export interface TrajectoriesStreamCallbacks {
  onStart: () => void;
  onTrajectory: (msg: SampledTrajectoryMessage) => void;
  onDone: (elapsedMs: number) => void;
  onCancel: () => void;
  onError: (err: Error) => void;
}

/**
 * Run a server-streaming PropagateTrajectories call. Honours `signal` for
 * cancellation; reports lifecycle events through `callbacks`.
 *
 * Returns when the stream ends, errors, or is aborted.
 */
export async function runPropagateTrajectories(
  req: PropagateRequest,
  cb: TrajectoriesStreamCallbacks,
  signal: AbortSignal,
): Promise<void> {
  if (signal.aborted) {
    cb.onCancel();
    return;
  }
  cb.onStart();
  const startedAt = performance.now();
  try {
    const stream = client.propagateTrajectories(req, { signal });
    for await (const msg of stream) {
      cb.onTrajectory(msg);
    }
    cb.onDone(performance.now() - startedAt);
  } catch (err) {
    if (signal.aborted) {
      cb.onCancel();
    } else {
      cb.onError(err instanceof Error ? err : new Error(String(err)));
    }
  }
}
