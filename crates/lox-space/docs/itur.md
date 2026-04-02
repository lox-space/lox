<!--
SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>

SPDX-License-Identifier: MPL-2.0
-->

# Atmospheric Propagation (ITU-R)

ITU-R P-series atmospheric propagation models for computing attenuation on
Earth-to-space radio paths. Based on the
[ITU-Rpy](https://github.com/iportillo/ITU-Rpy) library.

## Quick Start

Compute the total atmospheric attenuation for a ground station in Madrid
receiving at 14.25 GHz from a satellite at 30° elevation:

```python
import lox_space as lox

losses = lox.atmospheric_attenuation_slant_path(
    lat=40.4 * lox.deg,
    lon=-3.7 * lox.deg,
    frequency=14.25 * lox.GHz,
    elevation=30.0 * lox.deg,
    probability=0.01,       # exceeded 0.01% of the year
    diameter=1.2 * lox.m,   # antenna diameter
)

print(f"Rain:          {losses.rain}")
print(f"Gaseous:       {losses.gaseous}")
print(f"Cloud:         {losses.cloud}")
print(f"Scintillation: {losses.scintillation}")
print(f"Total:         {losses.atmospheric}")
```

The result is an `EnvironmentalLosses` object that plugs directly into
`LinkStats.calculate` for link budget analysis.

## Individual Models

Each ITU-R recommendation is also available as a standalone function.

### Rain Attenuation (P.618)

```python
a_rain = lox.rain_attenuation(
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
    rho=7.5,                      # water vapour density (g/m³)
    temperature=288.15 * lox.K,
)
```

### Cloud Attenuation (P.840)

```python
a_cloud = lox.cloud_attenuation(
    lat=40.4 * lox.deg,
    lon=-3.7 * lox.deg,
    elevation=30.0 * lox.deg,
    frequency=14.25 * lox.GHz,
    probability=1.0,
)
```

### Scintillation (P.618)

```python
a_scint = lox.scintillation_attenuation(
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
gamma_r = lox.rain_specific_attenuation(
    rain_rate=25.0,               # mm/h
    frequency=14.25 * lox.GHz,
    elevation=30.0 * lox.deg,
)
```

## Geophysical Data Lookups

### Topographic Altitude (P.1511)

```python
alt = lox.topographic_altitude(lat=27.99 * lox.deg, lon=86.93 * lox.deg)
print(f"Altitude: {alt.to_kilometers():.1f} km")
```

### Surface Temperature (P.1510)

```python
t = lox.surface_mean_temperature(lat=0.0 * lox.deg, lon=0.0 * lox.deg)
print(f"Temperature: {t.to_kelvin():.1f} K")
```

### Rainfall Rate (P.837)

```python
r = lox.rainfall_rate(lat=40.4 * lox.deg, lon=-3.7 * lox.deg, probability=0.01)
print(f"Rainfall rate: {r:.1f} mm/h")
```

### Rain Height (P.839)

```python
h = lox.rain_height(lat=40.4 * lox.deg, lon=-3.7 * lox.deg)
print(f"Rain height: {h.to_kilometers():.1f} km")
```

## Link Budget Integration

The `atmospheric_attenuation_slant_path` function returns an
`EnvironmentalLosses` object that can be passed directly to
`LinkStats.calculate`:

```python
losses = lox.atmospheric_attenuation_slant_path(
    lat=40.4 * lox.deg,
    lon=-3.7 * lox.deg,
    frequency=29.0 * lox.GHz,
    elevation=30.0 * lox.deg,
    probability=0.01,
    diameter=0.6 * lox.m,
)

stats = lox.LinkStats.calculate(
    tx_system=tx,
    rx_system=rx,
    channel=channel,
    range=1000.0 * lox.km,
    tx_angle=0.0 * lox.rad,
    rx_angle=0.0 * lox.rad,
    losses=losses,
)
```

## Data Files

Grid-based models (P.1511, P.1510, P.836, P.837, P.839, P.840, P.453)
require reference data from the ITU. This data is automatically downloaded
from PyPI and converted during the build process. No manual setup is needed.

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

::: lox_space.atmospheric_attenuation_slant_path
    options:
      show_source: false

---

::: lox_space.rain_attenuation
    options:
      show_source: false

---

::: lox_space.gaseous_attenuation_slant_path
    options:
      show_source: false

---

::: lox_space.cloud_attenuation
    options:
      show_source: false

---

::: lox_space.scintillation_attenuation
    options:
      show_source: false

---

::: lox_space.rain_specific_attenuation
    options:
      show_source: false

---

::: lox_space.topographic_altitude
    options:
      show_source: false

---

::: lox_space.surface_mean_temperature
    options:
      show_source: false

---

::: lox_space.rainfall_rate
    options:
      show_source: false

---

::: lox_space.rain_height
    options:
      show_source: false
