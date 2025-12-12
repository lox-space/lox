#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
#
# SPDX-License-Identifier: MPL-2.0

import os
import subprocess
import sys
from pathlib import Path


def run(cmd: list[str], env: dict[str, str] | None = None, check: bool = True) -> subprocess.CompletedProcess:
    """Run a command and return the result."""
    merged_env = {**os.environ, **(env or {})}
    print(f"$ {' '.join(cmd)}")
    return subprocess.run(cmd, env=merged_env, check=check)


def get_llvm_cov_env() -> dict[str, str]:
    """Get the environment variables from cargo llvm-cov show-env"""
    result = subprocess.run(
        ["cargo", "llvm-cov", "show-env"],
        capture_output=True,
        text=True,
        check=True,
    )

    env = {}
    for line in result.stdout.strip().split("\n"):
        # Lines are in format: export VAR='value'
        if "=" in line:
            key, value = line.split("=", 1)
            # Remove quotes
            value = value.strip("'\"")
            env[key] = value

    # Set CARGO_TARGET_DIR to match CARGO_LLVM_COV_TARGET_DIR
    if "CARGO_LLVM_COV_TARGET_DIR" in env:
        env["CARGO_TARGET_DIR"] = env["CARGO_LLVM_COV_TARGET_DIR"]

    env["CARGO_INCREMENTAL"] = "1"

    return env


def main():
    # Parse arguments
    report_args = sys.argv[1:] if len(sys.argv) > 1 else []

    # Get project root
    project_root = Path(__file__).parent.parent
    os.chdir(project_root)

    uv_project = "crates/lox-space"

    # Get llvm-cov environment
    print("Setting up coverage environment...")
    cov_env = get_llvm_cov_env()

    # Clean previous coverage data
    print("\nCleaning previous coverage data...")
    run(["cargo", "llvm-cov", "clean", "--workspace"], env=cov_env)

    # Run Rust tests
    print("\nRunning Rust tests...")
    run(["cargo", "nextest", "run", "--all-features"], env=cov_env)

    # Build PyO3 extension with maturin
    print("\nBuilding PyO3 extension...")
    run(["uv", "run", "--", "maturin", "develop", "--uv", "-m", f"{uv_project}/Cargo.toml"], env=cov_env)

    # Run Python tests with coverage
    # Use --no-sync to prevent uv from rebuilding the extension without coverage instrumentation
    print("\nRunning Python tests...")
    run(["uv", "run", "--no-sync", "--directory", uv_project, "--", "pytest", "--cov=lox_space", "--cov-report=xml"], env=cov_env)

    # Generate coverage report
    print("\nGenerating coverage report...")
    run(["cargo", "llvm-cov", "report"] + report_args, env=cov_env)


if __name__ == "__main__":
    main()
