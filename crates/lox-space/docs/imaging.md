<!--
SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>

SPDX-License-Identifier: MPL-2.0
-->

# Imaging access analysis

Lox computes per-(spacecraft, AOI) access windows for two sensor families:

- [Optical (passive) imaging](optical.md) — nadir-centred disk access geometry.
- [SAR (synthetic aperture radar)](sar.md) — side-looking annular access geometry.

Both share the same area-of-interest primitive ([`Aoi`](#areas-of-interest))
and the same result type ([`AccessResults`](#results)). See the individual
sensor pages for worked examples.

## Areas of interest

An `Aoi` is a closed geographic polygon defined by (longitude, latitude) pairs
in degrees. The first and last vertex must be the same:

```python
import lox_space as lox

# Rectangular bounding box around Rome
rome = lox.Aoi([(12.2, 41.7), (12.7, 41.7), (12.7, 42.1), (12.2, 42.1), (12.2, 41.7)])
```

You can also load an AOI from a GeoJSON string:

```python
sicily = lox.Aoi.from_geojson('{"type":"Polygon","coordinates":[[[13,37],[16,37],[16,39],[13,39],[13,37]]]}')
```

Spacecraft without a payload of the appropriate type are silently skipped
during analysis. Multiple AOIs can be passed to a single analysis run:

```python
analysis = lox.OpticalAccessAnalysis(
    scenario,
    aois=[("rome", rome), ("sicily", sicily)],
    step=30 * lox.seconds,
)
```

## Results

Both `OpticalAccessAnalysis` and `SarAccessAnalysis` return an `AccessResults`
object. Call `windows(spacecraft_name, aoi_name)` to retrieve the list of
access windows for a given spacecraft–AOI pair:

```python
results = analysis.compute()
for window in results.windows("S2A", "rome"):
    iv = window.interval()
    print(f"{iv.start()} → {iv.end()}  ({float(iv.duration()):.0f}s)")
```

### Pass direction

Each access window carries the spacecraft's pass direction
(`PassDirection.Ascending` or `PassDirection.Descending`) at the window
midpoint. Useful for InSAR coherence, change-detection workflows, and
disambiguating the two near-identical windows per orbit produced by
`SarPayload` with `LookSide.Either`.

```python
for window in results.windows("s1a", "europe"):
    print(window.interval(), window.direction())
```

For LEO orbits over non-polar AOIs the direction is essentially constant
through any single window (a typical LEO pass is short relative to a pole
crossing). The midpoint sample is representative.

---

::: lox_space.Aoi
    options:
      show_source: false

---

::: lox_space.AccessResults
    options:
      show_source: false
