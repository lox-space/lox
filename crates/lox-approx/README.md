<!--
SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>

SPDX-License-Identifier: MPL-2.0
-->

# lox-approx

Approximate equality testing for floating-point types, using both absolute and
relative tolerances. Two values are considered approximately equal when

```text
|a - b| ≤ max(atol, rtol * max(|a|, |b|))
```

which is the same closeness formula as Python's
[`math.isclose`](https://peps.python.org/pep-0485/), with stricter tolerance
validation: tolerances must be non-negative and finite.

## Features

- `approx_eq!` / `approx_ne!` macros returning `bool`, and `assert_approx_eq!` /
  `assert_approx_ne!` for tests with field-level failure diagnostics.
- `#[derive(ApproxEq)]` (behind the `derive` feature) for comparing structs
  field-by-field, reporting the exact failing field path.
- `no_std` support out of the box, with no math backend required.
- Optional `glam` feature for `DVec3` / `DMat3` impls.

The core (scalars, `Vec<T>`, `[T; N]`, and the derive) pulls in no third-party
dependencies; `glam` is opt-in. Because `glam` itself needs a math backend, a `no_std`
build with `glam` must also enable the `libm` feature (`--features glam,libm`); with the
default `std` feature no extra backend is needed.

## License

Licensed under the [Mozilla Public License v2.0](https://www.mozilla.org/en-US/MPL/2.0/).
