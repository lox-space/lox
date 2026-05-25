// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

import { describe, it, expect, vi, beforeEach } from "vitest";
import { runComputeAccess, type StreamCallbacks } from "./client";
import type { AccessRequest, AccessPairResult } from "@kerolox/proto-ts";

const mockServerStream = async function* () {
  yield { scId: "p0-s0", aoiId: "hormuz", windows: [], source: 1, comparatorId: "" } as unknown as AccessPairResult;
  yield { scId: "p0-s1", aoiId: "hormuz", windows: [], source: 1, comparatorId: "" } as unknown as AccessPairResult;
};

vi.mock("@connectrpc/connect-web", () => ({
  createConnectTransport: vi.fn(),
}));
vi.mock("@connectrpc/connect", () => ({
  createClient: vi.fn().mockImplementation(() => ({
    computeAccess: vi.fn().mockImplementation(() => mockServerStream()),
  })),
}));

describe("runComputeAccess", () => {
  let callbacks: StreamCallbacks;
  let pairs: AccessPairResult[];

  beforeEach(() => {
    pairs = [];
    callbacks = {
      onPair: (p) => pairs.push(p),
      onStart: vi.fn(),
      onDone: vi.fn(),
      onCancel: vi.fn(),
      onError: vi.fn(),
    };
  });

  it("invokes onStart, then onPair per chunk, then onDone", async () => {
    const req: AccessRequest = {} as AccessRequest;
    const ctl = new AbortController();
    await runComputeAccess(req, callbacks, ctl.signal);
    expect(callbacks.onStart).toHaveBeenCalledOnce();
    expect(pairs.length).toBe(2);
    expect(callbacks.onDone).toHaveBeenCalledOnce();
    expect(callbacks.onCancel).not.toHaveBeenCalled();
  });

  it("aborts mid-stream and reports onCancel", async () => {
    const req: AccessRequest = {} as AccessRequest;
    const ctl = new AbortController();
    ctl.abort();
    await runComputeAccess(req, callbacks, ctl.signal);
    expect(callbacks.onCancel).toHaveBeenCalledOnce();
    expect(callbacks.onDone).not.toHaveBeenCalled();
  });
});
