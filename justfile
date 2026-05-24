# SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
#
# SPDX-License-Identifier: MPL-2.0

export PYO3_PYTHON := `uv python find`

_default:
    just -l

build-pyo3 *FLAGS:
    uv run maturin develop --uv {{FLAGS}}

pytest *FLAGS:
    uv run pytest {{FLAGS}}

rstest *FLAGS:
    cargo nextest run --all-features --lib --bins --tests --examples {{FLAGS}}

doctest *FLAGS:
    cargo test --doc --all-features {{FLAGS}}

test: rstest doctest pytest

# Run tests with coverage (includes Python integration tests)
coverage *FLAGS:
    uv run --no-project tools/coverage.py {{FLAGS}}

lint-reuse *ARGS:
    uvx reuse lint {{ARGS}}

lint-clippy *ARGS:
    cargo clippy --all-features --all-targets {{ARGS}}

lint-rustfmt *ARGS:
    cargo fmt --check {{ARGS}}

lint: lint-reuse lint-clippy lint-rustfmt

# Add SPDX headers to new files
headers:
    uv run --no-project tools/add_spdx_headers.py

install-hooks:
    lefthook install

# Build Python documentation with zensical
docs-build:
    uv run --group docs zensical build

# Serve Python documentation with live reload
docs-serve:
    uv run --group docs zensical serve

# Serve Python documentation and open in browser
docs-open:
    uv run --group docs zensical serve --open
