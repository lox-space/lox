<!--
SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>

SPDX-License-Identifier: MPL-2.0
-->

# SAR access analysis

Lox models synthetic aperture radar (SAR) payloads via
[`SarPayload`](#sarpayload) and [`SarAccessAnalysis`](#saraccessanalysis).
The payload declares an angular envelope (look or incidence) and a looking
side; the analysis computes per-AOI access windows by scanning the
side-looking annular region under the ground track.

## Example: Sentinel-1 over Europe

```python
import lox_space as lox

sentinel1a = lox.SGP4("""\
SENTINEL-1A
1 39634U 14016A   26079.20000000  .00000050  00000+0  37000-4 0  9991
2 39634  98.1817 105.0000 0001300  90.0000 270.0000 14.59197557600008""")

# Sentinel-1 IW mode: ~29°–46° incidence, right-looking
payload = lox.SarPayload.with_incidence_angles(
    29.0 * lox.deg, 46.0 * lox.deg, lox.LookSide.Right
)
sc = lox.Spacecraft("s1a", sentinel1a, sar_payload=payload)

t0 = sentinel1a.time()
t1 = t0 + 6 * lox.hours
scenario = lox.Scenario(t0, t1, spacecraft=[sc])

europe = lox.Aoi(
    [(-10.0, 35.0), (20.0, 35.0), (20.0, 60.0), (-10.0, 60.0), (-10.0, 35.0)]
)

analysis = lox.SarAccessAnalysis(
    scenario,
    aois=[("europe", europe)],
    step=30 * lox.seconds,
)
results = analysis.compute()

for window in results.windows("s1a", "europe"):
    iv = window.interval()
    print(f"{iv.start()} → {iv.end()}  ({float(iv.duration()):.0f}s)")
    print(window.direction())
```

## Looking side

`SarPayload` selects the illuminated side via `LookSide`:

- `lox.LookSide.Right` — right-looking only (e.g. legacy fixed-side platforms such as Sentinel-1).
- `lox.LookSide.Left` — left-looking only.
- `lox.LookSide.Either` — roll-agile (e.g. modern small-sat SAR such as ICEYE or Capella).

## Angle conventions

Either constructor can be used depending on which angle convention is more
natural for a given sensor specification:

```python
# Look angle: measured at the spacecraft (off-nadir)
payload = lox.SarPayload.with_look_angles(
    20.0 * lox.deg, 45.0 * lox.deg, lox.LookSide.Either
)

# Incidence angle: measured at the ground point (off-vertical)
payload = lox.SarPayload.with_incidence_angles(
    22.0 * lox.deg, 46.0 * lox.deg, lox.LookSide.Right
)
```

Look and incidence angles are related via the spacecraft altitude and Earth's
mean radius:

```
sin(incidence) = sin(look) · (R + h) / R
```

The analysis converts between them per-sample using the actual altitude; no
reference altitude is baked into the payload.

## Limitations

- **Spherical Earth.** Both look↔incidence conversion and ground-range
  geometry use mean radius. The error is negligible below ~30° incidence and
  grows slowly at higher angles.
- **Large AOIs across the ground track.** The detector evaluates distance to
  the *nearest* AOI point. For AOIs wider than the inner annulus radius that
  straddle the ground track, this can produce false negatives — split such AOIs
  into smaller polygons or use point targets.
- **No squint, modes, or acquisition-quality outputs.** A single angular
  envelope per payload; no per-mode swath, NESZ, or resolution.

---

::: lox_space.SarPayload
    options:
      show_source: false

---

::: lox_space.LookSide
    options:
      show_source: false

---

::: lox_space.SarAccessAnalysis
    options:
      show_source: false
