# SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
# SPDX-License-Identifier: MPL-2.0

export UV_PROJECT := "crates/lox-space"

_default:
    just -l

build-pyo3 *FLAGS:
    uv run maturin develop --uv -m $UV_PROJECT/Cargo.toml {{FLAGS}}

pytest *FLAGS:
    uv run --directory $UV_PROJECT pytest {{FLAGS}}

rstest *FLAGS:
    cargo nextest run --all-features {{FLAGS}}

test: rstest pytest
