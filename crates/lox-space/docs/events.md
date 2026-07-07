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

The `VisibilityAnalysis` class computes visibility intervals between ground
stations and spacecraft, accounting for elevation constraints and optional
body occultation. A `Scenario` groups spacecraft, ground stations, and a
time interval together.

## Quick Example

```python
import lox_space as lox

# Visibility analysis
gs = lox.GroundStation("ESOC", ground_location, elevation_mask)
sc = lox.Spacecraft("ISS", lox.SGP4(tle))
scenario = lox.Scenario(start, end, spacecraft=[sc], ground_stations=[gs])
analysis = lox.VisibilityAnalysis(
    scenario,
    step=lox.TimeDelta(60),
    min_pass_duration=lox.TimeDelta(300),
)
results = analysis.compute(spk)

for iv in results.intervals("ESOC", "ISS"):
    print(f"Interval: {iv.start()} to {iv.end()}")

for p in results.passes("ESOC", "ISS"):
    print(f"Pass: {p.interval().start()} to {p.interval().end()}")
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

::: lox_space.VisibilityResults
    options:
      show_source: false

---

::: lox_space.PowerBudgetAnalysis
    options:
      show_source: false

---

::: lox_space.PowerBudgetResults
    options:
      show_source: false
