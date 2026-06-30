# SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
#
# SPDX-License-Identifier: MPL-2.0

_default:
    just -l

build-pyo3 *FLAGS:
    uv run maturin develop --uv {{FLAGS}}

# Pack the upstream `itur` Python wheel into target/lox-itur-data.npz.
#
# First time:
#   pip download --no-deps itur==0.4.0
#   just lox-itur-pack itur-0.4.0-py2.py3-none-any.whl
lox-itur-pack wheel:
    cargo run -p lox-itur --bin pack -- {{wheel}} target/lox-itur-data.npz

pytest *FLAGS:
    uv run pytest {{FLAGS}}

rstest *FLAGS:
    cargo nextest run --all-features --lib --bins --tests --examples {{FLAGS}}

doctest *FLAGS:
    cargo test --doc --all-features {{FLAGS}}

test: rstest doctest pytest

# Run Rust benchmarks
bench *FLAGS:
    cargo bench -p lox-space {{FLAGS}}

# Run Python benchmarks (build the wheel with `just build-pyo3 --release` first)
bench-py *FLAGS:
    uv run pytest --codspeed crates/lox-space/tests/test_*benchmark* {{FLAGS}}

# Run tests with coverage (includes Python integration tests)
coverage *FLAGS:
    uv run --no-project tools/coverage.py {{FLAGS}}

lint-reuse *ARGS:
    git ls-files -z | xargs -0 uvx --from 'reuse[charset-normalizer]' reuse lint-file {{ARGS}}

lint-clippy *ARGS:
    cargo clippy --all-features --all-targets {{ARGS}} -- -D warnings

# Bare-metal (no_std) clippy lint. Requires the thumbv7em-none-eabi target
# (`rustup target add thumbv7em-none-eabi`).
lint-clippy-embedded *ARGS:
    cargo clippy -p lox-core -p lox-approx -p lox-math -p lox-units -p lox-bodies -p lox-time --no-default-features --features libm --target thumbv7em-none-eabi {{ARGS}} -- -D warnings

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
