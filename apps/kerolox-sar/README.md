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

## What Phase 3 adds

- 3D globe (Threlte) and 2D equirectangular map, toggled in the
  viewport header.
- Per-satellite current position and ground track on both views,
  interpolated from a 30 s server-side sampled trajectory streamed over
  the `Kerolox::PropagateTrajectories` RPC.
- AOI polygons (Hormuz, Black Sea) rendered on both views.
- Shared `currentTime` playback transport (play / pause / scrub / rate
  1× / 10× / 60× / 600×).
- Earth rotation animated each frame via the IAU polynomial model
  (no IERS data dependency).
- Sampled trajectories cached client-side by a scenario hash; recomputed
  only when scenario inputs change.

## What Phase 4 adds

- **Trade-space sweep** — a "Trade study" tab fans out one `ComputeAccess`
  per swept parameter value (sats-per-plane / planes / altitude /
  inclination) with bounded concurrency, aggregates each completed
  scenario into a chosen metric (mean/median/max gap, window count, total
  access), and plots it live per AOI on a hand-rolled SVG chart. The sweep
  is cancellable mid-flight.
- **ICEYE comparison** — a "Compare vs ICEYE" toggle runs the real fielded
  ICEYE constellation through the same SAR access analysis. The engine
  resolves the bundled TLE snapshot to SGP4 propagators that join the same
  `Ensemble`; results stream back tagged `source: COMPARATOR`. The AOI tabs
  show user-vs-ICEYE revisit stats side by side, and the comparator
  satellites render in amber on both viewports (distinct from the
  per-plane user palette).

The ICEYE TLE snapshot lives at
`crates/kerolox-engine/data/comparators/iceye.tle` (CelesTrak, see the
`.license` sidecar). Refresh it by re-fetching CelesTrak's
`NAME=ICEYE` group query and replacing the file.
