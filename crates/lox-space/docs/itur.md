<!--
SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
SPDX-FileCopyrightText: 2026 Torkil Rein Gustavsen <torkilrg@ksat.no>

SPDX-License-Identifier: MPL-2.0
-->

# Atmospheric Propagation (ITU-R)

ITU-R P-series atmospheric propagation models for computing attenuation on
Earth-to-space radio paths. Based on the
[ITU-Rpy](https://github.com/iportillo/ITU-Rpy) library.

## Prerequisites

Grid-based models read reference data from a bundled `lox-itur-data.npz`
archive. Build it once from the upstream [`itur`](https://pypi.org/project/itur/)
Python wheel (see the [`lox-itur` README](https://github.com/lox-space/lox/tree/main/crates/lox-itur)
for the packaging workflow), then open it with `lox.ItuProvider`:

```python
import lox_space as lox

provider = lox.ItuProvider("lox-itur-data.npz")
```

A single provider can be reused for all lookups; grids are loaded lazily and
cached on first access.

## Quick Start

Compute the total atmospheric attenuation for a ground station in Madrid
receiving at 14.25 GHz from a satellite at 30° elevation:

```python
losses = provider.propagation_losses(
    lat=40.4 * lox.deg,
    lon=-3.7 * lox.deg,
    frequency=14.25 * lox.GHz,
    elevation=30.0 * lox.deg,
    probability=0.01,  # exceeded 0.01% of the year
    diameter=1.2 * lox.m,  # antenna diameter
)

for label, value, absorptive in losses.lines:
    print(f"{label}: {value}")
print(f"Total:      {losses.total()}")
print(f"Absorptive: {losses.absorptive()}")
```

The rain, gaseous, and cloud lines carry the raw model outputs; the
combination residual of the ITU-R P.618 total is booked on the
scintillation line, so `total()` equals the combined attenuation and
`absorptive()` is exactly the part that raises the antenna noise
temperature (used for the rain-degraded G/T on downlinks).

For link budgets with known loss values, build `PropagationLosses`
directly:

```python
losses = lox.PropagationLosses(rain=2.0 * lox.dB, gaseous=0.5 * lox.dB)
losses = lox.PropagationLosses.none()
```

The result plugs directly into `LinkBudget` for link budget analysis.

## Individual Models

Each ITU-R recommendation is also available as a standalone function.

### Rain Attenuation (P.618)

```python
a_rain = provider.rain_attenuation(
    lat=40.4 * lox.deg,
    lon=-3.7 * lox.deg,
    frequency=14.25 * lox.GHz,
    elevation=30.0 * lox.deg,
    probability=0.01,
)
```

### Gaseous Attenuation (P.676)

```python
a_oxygen, a_water = lox.gaseous_attenuation_slant_path(
    frequency=14.25 * lox.GHz,
    elevation=30.0 * lox.deg,
    pressure=1013.25 * lox.hPa,
    rho_g_per_m3=7.5,  # water vapour density (g/m³)
    temperature=288.15 * lox.K,
)
```

### Cloud Attenuation (P.840)

```python
a_cloud = provider.cloud_attenuation(
    lat=40.4 * lox.deg,
    lon=-3.7 * lox.deg,
    elevation=30.0 * lox.deg,
    frequency=14.25 * lox.GHz,
    probability=1.0,
)
```

### Scintillation (P.618)

```python
a_scint = provider.scintillation_attenuation(
    frequency=14.25 * lox.GHz,
    elevation=30.0 * lox.deg,
    probability=0.01,
    diameter=1.2 * lox.m,
    lat=40.4 * lox.deg,
    lon=-3.7 * lox.deg,
)
```

### Specific Rain Attenuation (P.838)

```python
gamma_r = lox.rain_specific_attenuation_db_per_km(
    rain_rate_mm_per_hour=25.0,  # mm/h
    frequency=14.25 * lox.GHz,
    elevation=30.0 * lox.deg,
)
```

## Geophysical Data Lookups

### Topographic Altitude (P.1511)

```python
alt = provider.topographic_altitude(lat=27.99 * lox.deg, lon=86.93 * lox.deg)
print(f"Altitude: {alt.to_kilometers():.1f} km")
```

### Surface Temperature (P.1510)

```python
t = provider.surface_mean_temperature(lat=0.0 * lox.deg, lon=0.0 * lox.deg)
print(f"Temperature: {t.to_kelvin():.1f} K")
```

### Rainfall Rate (P.837)

```python
r = provider.rainfall_rate_mm_per_hour(
    lat=40.4 * lox.deg, lon=-3.7 * lox.deg, probability=0.01
)
print(f"Rainfall rate: {r:.1f} mm/h")
```

### Rain Height (P.839)

```python
h = provider.rain_height(lat=40.4 * lox.deg, lon=-3.7 * lox.deg)
print(f"Rain height: {h.to_kilometers():.1f} km")
```

## Link Budget Integration

`PropagationLosses` can be passed directly to `LinkBudget`, where
`tx` is a `TxChain` or `EirpModel` and `rx` is an `RxChain` or `GtModel` (see
[Communications](comms.md)):

```python
losses = provider.propagation_losses(
    lat=40.4 * lox.deg,
    lon=-3.7 * lox.deg,
    frequency=29.0 * lox.GHz,
    elevation=30.0 * lox.deg,
    probability=0.01,
    diameter=0.6 * lox.m,
)

budget = lox.LinkBudget(
    tx,
    rx,
    carrier=29.0 * lox.GHz,
    range=1000.0 * lox.km,
    losses=losses,
)
```

## Elevation Profiles

`propagation_losses` answers for one elevation. Along a pass the elevation —
and with it the loss — changes continuously, so the provider can tabulate the
losses over a grid of elevations into an `ElevationProfile`, which interpolates
between its samples:

```python
profile = provider.elevation_profile(
    lat=78.23 * lox.deg,
    lon=15.39 * lox.deg,
    frequency=26.0 * lox.GHz,
    probability=5.0,
    diameter=3.0 * lox.m,
)

for elevation in (5.0, 10.0, 30.0):
    total = profile.at(elevation * lox.deg).total()
    print(f"{elevation:4.0f} deg: {total.to_decibels():.2f} dB")
```

The default grid has 14 elevations from 5° to 90°, denser towards the horizon;
interpolation on it stays within about 1.5 % of the direct computation up to
80°. Pass `elevations` for a grid of your own.

`site_propagation` builds one profile per carrier in a single call — typically
the downlink and the uplink carrier of a ground station. The result plugs into
[`LinkBudgetAnalysis`](link-budget.md#losses-that-follow-the-elevation):

```python
site = provider.site_propagation(
    lat=78.23 * lox.deg,
    lon=15.39 * lox.deg,
    carriers=[26.0 * lox.GHz, 7.2 * lox.GHz],
    probability=5.0,
    diameter=3.0 * lox.m,
)
```

## Data Files

Grid-based models (P.453, P.618, P.836, P.837, P.839, P.840, P.1510, P.1511)
require reference data from the ITU, distributed via the upstream `itur`
Python package. Build a `lox-itur-data.npz` bundle once and open it with
`lox.ItuProvider`. See the
[`lox-itur` README](https://github.com/lox-space/lox/tree/main/crates/lox-itur)
for the packaging workflow.

## Implemented Recommendations

| Recommendation | Description |
|----------------|-------------|
| P.453-13 | Radio refractive index |
| P.618-14 | Earth-space propagation (rain, scintillation, XPD) |
| P.676-13 | Gaseous attenuation (O₂ + H₂O) |
| P.835-6 | Reference standard atmospheres |
| P.836-6 | Water vapour surface density |
| P.837-7 | Rainfall rate |
| P.838-3 | Specific rain attenuation |
| P.839-4 | Rain height |
| P.840-9 | Cloud and fog attenuation |
| P.1510-1 | Surface temperature |
| P.1511-2 | Topographic altitude |

---

::: lox_space.ItuProvider
    options:
      show_source: false

---

::: lox_space.PropagationLosses
    options:
      show_source: false

---

::: lox_space.ElevationProfile
    options:
      show_source: false

---

::: lox_space.SitePropagation
    options:
      show_source: false

---

::: lox_space.gaseous_attenuation_slant_path
    options:
      show_source: false

---

::: lox_space.rain_specific_attenuation_db_per_km
    options:
      show_source: false
