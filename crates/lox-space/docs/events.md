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

The `visibility` function computes visibility windows between a ground station
and a spacecraft, accounting for elevation constraints.

## Quick Example

```python
import lox_space as lox

# Find events on a trajectory
def altitude(state):
    """Returns altitude above reference radius."""
    r = (state.position()[0]**2 + state.position()[1]**2 + state.position()[2]**2)**0.5
    return r - 6378.0  # km above Earth radius

events = trajectory.find_events(altitude)
for event in events:
    print(f"{event.crossing()} crossing at {event.time()}")

# Find time windows
windows = trajectory.find_windows(altitude)
for w in windows:
    print(f"Window: {w.start()} to {w.end()}, duration: {w.duration()}")

# Visibility analysis
passes = lox.visibility(
    times=times,
    gs=ground_station,
    mask=elevation_mask,
    sc=trajectory,
    ephemeris=spk,
)

for p in passes:
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

::: lox_space.visibility
    options:
      show_source: false

---

::: lox_space.visibility_all
    options:
      show_source: false
