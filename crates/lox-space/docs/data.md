<!--
SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>

SPDX-License-Identifier: MPL-2.0
-->

# Data Providers

External data sources for Earth orientation and interpolation.

## Earth Orientation Parameters

Earth Orientation Parameters (EOP) are required for accurate transformations
involving UT1 (Earth rotation) and polar motion corrections.

## Quick Example

```python
import lox_space as lox

# Load EOP data
eop = lox.EOPProvider("/path/to/finals2000A.all.csv")

# Use with time scale conversions
t_tai = lox.Time("TAI", 2024, 1, 1)
t_ut1 = t_tai.to_scale("UT1", provider=eop)

# Use with frame transformations
state_itrf = state.to_frame(lox.Frame("ITRF"), provider=eop)

# Custom interpolation series
import numpy as np
x = [0.0, 1.0, 2.0, 3.0]
y = [0.0, 1.0, 4.0, 9.0]

# Linear interpolation (default)
series = lox.Series(x, y)
print(series.interpolate(1.5))  # ~2.5

# Cubic spline interpolation
series = lox.Series(x, y, method="cubic_spline")
print(series.interpolate(1.5))  # ~2.25
```

---

::: lox_space.EOPProvider
    options:
      show_source: false

---

::: lox_space.Series
    options:
      show_source: false
