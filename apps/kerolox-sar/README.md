<!--
  SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
  SPDX-License-Identifier: MPL-2.0
-->

# kerolox-sar

Vertical slice of the Kerolox SAR constellation sizing tool. This app is the
frontend half — a SvelteKit app with a scenario form that runs the WASM
Walker delta builder and shows the satellite list. The compute engine and
streaming access analysis arrive in later phases.

## Run

```bash
# One-time: build the WASM consumer package.
wasm-pack build packages/lox-space-wasm --target web

# Install JS deps.
pnpm install

# Dev server with HMR.
pnpm --filter kerolox-sar dev
```

Open <http://localhost:5173>.

## Test

```bash
pnpm --filter kerolox-sar test:unit   # Vitest
pnpm --filter kerolox-sar check       # svelte-check + tsc
pnpm --filter kerolox-sar build       # required before test:e2e
pnpm --filter kerolox-sar test:e2e    # Playwright — uses `vite preview`, so build first
```

The E2E test downloads Chromium via `playwright install chromium` on first
run. On a locked-down sandbox you may need to allow `cdn.playwright.dev` and
`playwright.download.prss.microsoft.com` first.

## What this phase ships

- Scenario form (start time, duration, Walker T/P/F/altitude/inclination,
  SAR look side + min/max incidence).
- In-browser Walker delta evaluation via `@lox-space/wasm`.
- Satellites tab with the orbital elements of every satellite in the design.
- Status pill (idle only).
- JSON dump in the viewport area (the Phase 3 globe / map lives here).

## Not yet shipped (later phases)

- Compute engine + streaming access analysis (Phase 2).
- 3D globe and 2D map viewports, ground tracks, play/pause animation (Phase 3).
- Trade-space sweep + ICEYE comparator (Phase 4).
