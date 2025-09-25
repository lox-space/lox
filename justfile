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
