<!--
SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>

SPDX-License-Identifier: MPL-2.0
-->

# Events & Visibility

Event detection and visibility analysis.

## Event Detection

Events are detected when a function crosses zero. The `crossing` property
indicates the direction:

- `"up"`: Function crosses from negative to positive
- `"down"`: Function crosses from positive to negative

## Visibility Analysis

`VisibilityAnalysis` computes ground-station-to-spacecraft passes, accounting
for elevation constraints and optional body occultation. `InterSatelliteAnalysis`
is its spacecraft-to-spacecraft counterpart — a separate class because sat-to-sat
contacts have no ground observables, so they yield `Window`s rather than
`Pass`es. A `Scenario` groups spacecraft, ground stations, and a time interval
together.

Each analysis offers two entry points. `single()` computes one target and raises
on failure; `run()` fans out over every target and returns per-target results, so
one unresolvable ephemeris cannot sink a batch.

## Quick Example

```python
import lox_space as lox

# Visibility analysis
gs = lox.GroundStation("ESOC", ground_location, elevation_mask)
sc = lox.Spacecraft("ISS", lox.SGP4(tle))
scenario = lox.Scenario(start, end, spacecraft=[sc], ground_stations=[gs])
analysis = lox.VisibilityAnalysis(
    scenario,
    ephemeris=spk,
    step=lox.TimeDelta(60),
    min_pass_duration=lox.TimeDelta(300),
)

# One pair, with observables.
for p in analysis.single(gs, sc):
    print(f"Pass: {p.interval().start()} to {p.interval().end()}")

# One pair, timing only — about a third cheaper, since nothing samples
# azimuth, elevation, range, or range rate.
for w in analysis.windows(gs, sc):
    print(f"Window: {w.start} to {w.end}")

# Every pair, keyed. Under `parallel=True` targets finish out of order, which
# is why the key travels with the result instead of being inferred by position.
run = analysis.run()
for (station, spacecraft), passes in run.passes.items():
    print(f"{station} ↔ {spacecraft}: {len(passes)} passes")
for pair, message in run.errors.items():
    print(f"{pair} failed: {message}")
```

Inter-satellite contacts use the same shape:

```python
isl = lox.InterSatelliteAnalysis(scenario, ephemeris=spk, max_range=5000 * lox.km)
for w in isl.single(sc_a, sc_b):
    print(f"Contact: {w.start} to {w.end} ({float(w.duration):.0f}s)")
```

---

::: lox_space.Event
    options:
      show_source: false

---

::: lox_space.Interval
    options:
      show_source: false

---

::: lox_space.intersect_intervals
    options:
      show_source: false

---

::: lox_space.union_intervals
    options:
      show_source: false

---

::: lox_space.complement_intervals
    options:
      show_source: false

---

::: lox_space.GroundStation
    options:
      show_source: false

---

::: lox_space.Spacecraft
    options:
      show_source: false

---

::: lox_space.Scenario
    options:
      show_source: false

---

::: lox_space.Ensemble
    options:
      show_source: false

---

::: lox_space.VisibilityAnalysis
    options:
      show_source: false

---

::: lox_space.VisibilityRun
    options:
      show_source: false

---

::: lox_space.InterSatelliteAnalysis
    options:
      show_source: false

---

::: lox_space.InterSatelliteRun
    options:
      show_source: false

---

::: lox_space.Window
    options:
      show_source: false

---

::: lox_space.PowerBudgetAnalysis
    options:
      show_source: false

---

::: lox_space.PowerBudgetRun
    options:
      show_source: false

---

::: lox_space.SpacecraftPower
    options:
      show_source: false

---

::: lox_space.Eclipse
    options:
      show_source: false
