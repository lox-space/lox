<!--
SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>

SPDX-License-Identifier: MPL-2.0
-->

# Optical imaging access

Lox models passive (electro-optical) payloads via [`OpticalPayload`](#opticalpayload)
and [`OpticalAccessAnalysis`](#opticalaccessanalysis). The payload declares a
nadir-centred swath width and an optional off-nadir pointing capability; the
analysis computes per-AOI access windows by scanning the circular ground
footprint under the satellite.

## Sensor models

### Nadir-only

For sensors that image straight down (e.g. Sentinel-2), the accessible ground
range equals half the swath width:

```python
import lox_space as lox

payload = lox.OpticalPayload.nadir_only(290.0 * lox.km)
```

### Off-nadir pointing

For satellites that can slew away from nadir to image targets, the accessible
range is the sum of the off-nadir ground range and half the swath width:

```python
payload = lox.OpticalPayload.off_nadir(
    swath_width=20.0 * lox.km,
    max_off_nadir=30.0 * lox.deg,
)
```

The off-nadir ground range is computed geometrically from the spacecraft
altitude, the maximum off-nadir angle, and the body's mean radius.

## Attaching payloads to spacecraft

Pass the payload via the `optical_payload` keyword argument when constructing
a `Spacecraft`. Spacecraft without a payload are silently skipped during
analysis:

```python
# With payload — included in optical access analysis
sc1 = lox.Spacecraft("imager", orbit, optical_payload=payload)

# Without payload — skipped
sc2 = lox.Spacecraft("relay", orbit)

scenario = lox.Scenario(t0, t1, spacecraft=[sc1, sc2])
# Only sc1 will produce imaging windows
```

## Example: Sentinel-2 over Europe

```python
import lox_space as lox

sentinel2a = lox.SGP4("""\
SENTINEL-2A
1 40697U 15028A   26079.19377485 -.00000072  00000+0 -11026-4 0  9994
2 40697  98.5642 155.3327 0001269  98.1407 261.9920 14.30816376561005""")

payload = lox.OpticalPayload.nadir_only(290.0 * lox.km)
sc = lox.Spacecraft("S2A", sentinel2a, optical_payload=payload)

t0 = sentinel2a.time()
t1 = t0 + 6 * lox.hours
scenario = lox.Scenario(t0, t1, spacecraft=[sc])

europe = lox.Aoi(
    [(-10.0, 35.0), (20.0, 35.0), (20.0, 60.0), (-10.0, 60.0), (-10.0, 35.0)]
)

analysis = lox.OpticalAccessAnalysis(
    scenario,
    aois=[("europe", europe)],
    step=30 * lox.seconds,
)
results = analysis.compute()

for iv in results.intervals("S2A", "europe"):
    print(f"{iv.start()} → {iv.end()}  ({float(iv.duration()):.0f}s)")
```

---

::: lox_space.OpticalPayload
    options:
      show_source: false

---

::: lox_space.OpticalAccessAnalysis
    options:
      show_source: false
