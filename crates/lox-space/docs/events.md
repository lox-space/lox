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
body occultation.

## Quick Example

```python
import lox_space as lox

# Find events on a trajectory
def altitude(state):
    """Returns altitude above reference radius."""
    x, y, z = state.position()
    r = (float(x)**2 + float(y)**2 + float(z)**2)**0.5
    return r - 6378000.0  # meters above Earth radius

step = lox.TimeDelta(10.0)  # 10-second step size
events = trajectory.find_events(altitude, step)
for event in events:
    print(f"{event.crossing()} crossing at {event.time()}")

# Find time intervals
intervals = trajectory.find_windows(altitude, step)
for iv in intervals:
    print(f"Interval: {iv.start()} to {iv.end()}, duration: {iv.duration()}")

# Visibility analysis
gs = lox.GroundAsset("ESOC", ground_location, elevation_mask)
sc = lox.SpaceAsset("ISS", trajectory)
analysis = lox.VisibilityAnalysis(
    [gs], [sc],
    step=lox.TimeDelta(60),
    min_pass_duration=lox.TimeDelta(300),
)
results = analysis.compute(start, end, spk)

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

::: lox_space.find_events
    options:
      show_source: false

---

::: lox_space.find_windows
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

::: lox_space.GroundAsset
    options:
      show_source: false

---

::: lox_space.SpaceAsset
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
