# SPDX-FileCopyrightText: 2024 Helge Eichhorn <git@helgeeichhorn.de>
#
# SPDX-License-Identifier: MPL-2.0

_default:
    just -l

# Pyodide release to build the browser wheel against. Pins CPython, Emscripten
# and Rust; `pyodide xbuildenv search` lists the compatible releases.
pyodide_version := "314.0.5"
pyodide_python := "3.14"

# Tests that compare against packages Pyodide does not ship.
pyodide_test_excludes := "--ignore=crates/lox-space/tests/test_astropy_frames.py \
    --ignore=crates/lox-space/tests/test_frames.py \
    --ignore=crates/lox-space/tests/test_teme_frame.py \
    --ignore=crates/lox-space/tests/test_spacelink_comms.py"

build-pyo3 *FLAGS:
    uv run maturin develop --uv {{FLAGS}}

# One-off setup for the Pyodide wheel build (~2 GB download).
pyodide-setup:
    uv venv --python {{pyodide_python}} .venv-pyodide-build
    VIRTUAL_ENV=.venv-pyodide-build uv pip install pyodide-build
    .venv-pyodide-build/bin/pyodide xbuildenv install {{pyodide_version}}
    rustup toolchain install "$(.venv-pyodide-build/bin/pyodide config get rust_toolchain)" \
        --target wasm32-unknown-emscripten --profile minimal

# Build the Pyodide wheel into dist-pyodide/.
build-pyodide *FLAGS:
    rm -rf dist-pyodide
    RUSTUP_TOOLCHAIN="$(.venv-pyodide-build/bin/pyodide config get rust_toolchain)" \
        .venv-pyodide-build/bin/pyodide build -o dist-pyodide {{FLAGS}}

# Run the Python tests against the Pyodide wheel in Node.
pytest-pyodide *FLAGS:
    PATH="$PWD/.venv-pyodide-build/bin:$PATH" \
        .venv-pyodide-build/bin/pyodide venv --clear .venv-pyodide
    .venv-pyodide/bin/pip install pytest dist-pyodide/*.whl
    .venv-pyodide/bin/python -m pytest -m "not benchmark" {{pyodide_test_excludes}} {{FLAGS}}

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
