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

The `VisibilityAnalysis` class computes visibility windows between ground
stations and spacecraft, accounting for elevation constraints and optional
body occultation.

## Quick Example

```python
import lox_space as lox

# Find events on a trajectory
def altitude(state):
    """Returns altitude above reference radius."""
    r = (state.position()[0]**2 + state.position()[1]**2 + state.position()[2]**2)**0.5
    return r - 6378.0  # km above Earth radius

step = lox.TimeDelta(10.0)  # 10-second step size
events = trajectory.find_events(altitude, step)
for event in events:
    print(f"{event.crossing()} crossing at {event.time()}")

# Find time windows
windows = trajectory.find_windows(altitude, step)
for w in windows:
    print(f"Window: {w.start()} to {w.end()}, duration: {w.duration()}")

# Visibility analysis
gs = lox.GroundAsset("ESOC", ground_location, elevation_mask)
sc = lox.SpaceAsset("ISS", trajectory)
analysis = lox.VisibilityAnalysis([gs], [sc], step=60.0)
results = analysis.compute(start, end, spk)

for w in results.intervals("ESOC", "ISS"):
    print(f"Window: {w.start()} to {w.end()}")

for p in results.passes("ESOC", "ISS"):
    print(f"Pass: {p.window().start()} to {p.window().end()}")
```

---

::: lox_space.Event
    options:
      show_source: false

---

::: lox_space.Window
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
