<!--
SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>

SPDX-License-Identifier: MPL-2.0
-->

# Browser & Pyodide

Lox publishes an Emscripten wheel alongside the native ones, so the full Python
API runs in the browser under [Pyodide](https://pyodide.org) — in a plain web
page, in JupyterLite, or in any other Pyodide host.

## Installing

Inside a Pyodide runtime:

```python
import micropip

await micropip.install("lox-space")

import lox_space as lox
```

The wheel is tagged `pyemscripten_2026_0_wasm32`, which is the ABI of Pyodide
`314.x` (CPython 3.14). Pyodide releases with a different ABI version need a
wheel built against that release.

In a web page:

```html
<script type="module">
  import { loadPyodide } from "https://cdn.jsdelivr.net/pyodide/v314.0.5/full/pyodide.mjs";

  const pyodide = await loadPyodide();
  await pyodide.loadPackage("micropip");
  await pyodide.runPythonAsync(`
    import micropip
    await micropip.install("lox-space")

    import lox_space as lox

    t = lox.Time("TAI", 2026, 8, 24, 12, 0, 0.0)
    state = lox.Cartesian(
        time=t,
        x=6678.0 * lox.km, y=0.0 * lox.km, z=0.0 * lox.km,
        vx=0.0 * lox.km_per_s, vy=7.73 * lox.km_per_s, vz=0.0 * lox.km_per_s,
    )
    print(state.to_keplerian().orbital_period())
  `);
</script>
```

## Differences from the native wheel

The API is the same — nothing is feature-gated out of the browser build — but
two properties of the runtime are worth knowing about.

**Everything runs on one thread.** Pyodide's CPython is built without pthreads,
so the fan-out that the native build uses for visibility, power, imaging access
and scenario propagation runs sequentially here. Results are identical; large
scenarios take proportionally longer.

**Files live in a virtual filesystem.** Anything that takes a path — [`SPK`](frames.md),
[`EOPProvider`](data.md), [`ItuProvider`](itur.md) — reads through Emscripten's
in-memory filesystem, so fetch the file and write it there first:

```python
import lox_space as lox
from pyodide.http import pyfetch

response = await pyfetch("https://example.org/kernels/de440s.bsp")
with open("de440s.bsp", "wb") as f:
    f.write(await response.bytes())

ephemeris = lox.SPK("de440s.bsp")
```

Budget for the download: `de440s.bsp` is ~31 MB, and the packed ITU-R bundle
that [`ItuProvider`](itur.md) expects is ~490 MB — large enough that serving a
pre-trimmed subset is usually the better option in a browser.

## Building the wheel

The pinned Pyodide release — which in turn pins the CPython, Emscripten and Rust
versions of the build — lives in `pyodide_version` in the `justfile`. `pyodide
xbuildenv search` lists the releases compatible with a given host Python.

```bash
just pyodide-setup     # one-off: cross-build environment + pinned Rust toolchain (~2 GB)
just build-pyodide     # wheel into dist-pyodide/
just pytest-pyodide    # run the Python test suite in Node
```

`just pytest-pyodide` skips the tests that compare Lox against packages Pyodide
does not ship (astropy, skyfield, spiceypy, spacelink); everything else runs
against the wasm wheel.

`build-pyodide` sets `RUSTUP_TOOLCHAIN` itself, because `pyodide build` exports
the toolchain it pins without selecting it — left alone, the build would use the
default toolchain, which need not have the Emscripten target installed.
