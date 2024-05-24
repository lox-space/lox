_default:
    just -l

build-pyo3:
    maturin develop -E dev -m crates/lox-space/Cargo.toml

pytest *FLAGS: build-pyo3
    pytest {{FLAGS}}

rstest *FLAGS:
    cargo nextest run --all-features {{FLAGS}}

test: rstest pytest
