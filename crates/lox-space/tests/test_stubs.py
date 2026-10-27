# SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
#
# SPDX-License-Identifier: MPL-2.0

"""The type stub must match the runtime module."""

import os
import pathlib
import subprocess
import sys

import pytest

pytest.importorskip("mypy", reason="stubtest needs mypy (dev dependency group)")

ROOT = pathlib.Path(__file__).parents[3]


def test_stubtest():
    # Runs against the `lox_space` installed in this interpreter, i.e. exactly
    # the build the rest of the suite tests. `MYPYPATH` points stubtest at the
    # repository's stub rather than the copy bundled into that build. Since the
    # stub declares `__all__`, this also catches runtime exports it is missing.
    result = subprocess.run(
        [
            sys.executable,
            "-m",
            "mypy.stubtest",
            "lox_space",
            "--allowlist",
            str(ROOT / "stubtest-allowlist.txt"),
        ],
        cwd=ROOT,
        env={**os.environ, "MYPYPATH": str(ROOT)},
        capture_output=True,
        text=True,
    )
    assert result.returncode == 0, result.stdout + result.stderr
