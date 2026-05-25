<!--
  SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
  SPDX-License-Identifier: MPL-2.0
-->

# kerolox-sar

Vertical slice of the Kerolox SAR constellation sizing tool. SvelteKit
frontend talking to the `kerolox-engine` Rust binary over ConnectRPC.

## Run (two-process dev loop)

In one terminal — start the compute engine:

```bash
just kerolox-engine-dev
# or: cargo run -p kerolox-engine
```

It binds to `127.0.0.1:8080` by default (`KEROLOX_ADDR` env var
overrides). Confirm with:

```bash
curl http://127.0.0.1:8080/health    # -> OK
```

In another terminal — build WASM (if not done) and start SvelteKit:

```bash
wasm-pack build packages/lox-space-wasm --target web   # one-time
pnpm install
pnpm --filter kerolox-sar dev
```

Open <http://localhost:5173>. As you edit any input in the form, the
satellites table updates immediately (WASM), and a debounced
ComputeAccess stream re-runs on the engine; results land in the
Hormuz / Black Sea tabs as they arrive.

## Test

```bash
pnpm --filter kerolox-sar test:unit   # Vitest (scenario, walker, access, rpc-client)
pnpm --filter kerolox-sar check       # svelte-check + tsc
pnpm --filter kerolox-sar build       # required before test:e2e
pnpm --filter kerolox-sar test:e2e    # Playwright (needs engine running)
```

## What this phase ships (Phase 2)

- `kerolox-engine` binary serving `Kerolox::ComputeAccess` over
  ConnectRPC, streaming `AccessPairResult` per (satellite, AOI) pair.
- Bundled AOI library (Hormuz, Black Sea) as GeoJSON polygons.
- Frontend Connect-Web client with debounced auto-recompute and
  AbortController-driven cancellation.
- Per-AOI tabs with revisit-time stats and streaming window list.
- Status pill with widened states (idle / computing / done / error /
  cancelled) and `done · X.X s` compute-time display.
- Streaming-append row animation, `N/M pairs` counter, visible
  cancellation transition.

## Not yet shipped (later phases)

- 3D globe and 2D map viewports, ground tracks, play/pause animation
  (Phase 3).
- Trade-space sweep + ICEYE comparator (Phase 4).
