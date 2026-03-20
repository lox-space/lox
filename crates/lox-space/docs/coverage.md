<!--
SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>

SPDX-License-Identifier: MPL-2.0
-->

# Coverage

Compute when spacecraft can observe ground areas of interest.

## Overview

The imaging analysis module determines when a spacecraft's sensor footprint
intersects a geographic polygon (AOI). It works by computing the sub-satellite
point at each time step and checking whether the sensor's ground-accessible
range (swath width plus off-nadir pointing capability) reaches the AOI.

The analysis integrates with the existing event detection framework, so imaging
windows are computed via root-finding on a continuous signal function — the same
approach used for visibility analysis.

## Quick Example

```python
import lox_space as lox

# 1. Define the sensor payload
payload = lox.ImagingPayload.nadir_only(290.0 * lox.km)       # Sentinel-2-like
# payload = lox.ImagingPayload.off_nadir(20 * lox.km, 30 * lox.deg)  # off-nadir pointing

# 2. Create spacecraft with the payload attached
sc = lox.Spacecraft("S2A", sgp4_propagator, imaging_payload=payload)

# 3. Build a scenario
scenario = lox.Scenario(t0, t1, spacecraft=[sc])

# 4. Define areas of interest
rome = lox.Aoi([(12.2, 41.7), (12.7, 41.7), (12.7, 42.1), (12.2, 42.1), (12.2, 41.7)])

# Or from GeoJSON
sicily = lox.Aoi.from_geojson('{"type":"Polygon","coordinates":[[[13,37],[16,37],[16,39],[13,39],[13,37]]]}')

# 5. Run the analysis
analysis = lox.ImagingAnalysis(
    scenario,
    aois=[("rome", rome), ("sicily", sicily)],
    step=30 * lox.seconds,
)
results = analysis.compute()

# 6. Inspect results
for iv in results.intervals("S2A", "rome"):
    print(f"{iv.start()} → {iv.end()}  ({float(iv.duration()):.0f}s)")
```

## Sensor Models

### Nadir-Only

For sensors that image straight down (e.g. Sentinel-2), the accessible ground
range equals half the swath width:

```python
payload = lox.ImagingPayload.nadir_only(290.0 * lox.km)
```

### Off-Nadir Pointing

For satellites that can point away from nadir to image targets, the accessible
range is the sum of the off-nadir ground range and half the swath width:

```python
payload = lox.ImagingPayload.off_nadir(
    swath_width=20.0 * lox.km,
    max_off_nadir=30.0 * lox.deg,
)
```

The off-nadir ground range is computed geometrically from the spacecraft
altitude, the maximum off-nadir angle, and the body's mean radius.

## Attaching Payloads to Spacecraft

Spacecraft without a payload are silently skipped during analysis:

```python
# With payload — will be included in imaging analysis
sc1 = lox.Spacecraft("imager", orbit, imaging_payload=payload)

# Without payload — will be skipped
sc2 = lox.Spacecraft("relay", orbit)

scenario = lox.Scenario(t0, t1, spacecraft=[sc1, sc2])
# Only sc1 will produce imaging windows
```

---

::: lox_space.Aoi
    options:
      show_source: false

---

::: lox_space.ImagingPayload
    options:
      show_source: false

---

::: lox_space.ImagingAnalysis
    options:
      show_source: false

---

::: lox_space.ImagingResults
    options:
      show_source: false
