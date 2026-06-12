# SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
#
# SPDX-License-Identifier: MPL-2.0

"""The type stub must cover every public runtime export."""

import ast
import pathlib

import lox_space as lox


def test_stub_covers_all_exports():
    stub = pathlib.Path(__file__).parents[3] / "lox_space.pyi"
    tree = ast.parse(stub.read_text())
    stub_names = {
        node.name
        for node in tree.body
        if isinstance(node, (ast.ClassDef, ast.FunctionDef))
    }
    stub_names |= {
        target.id
        for node in tree.body
        if isinstance(node, ast.Assign)
        for target in node.targets
        if isinstance(target, ast.Name)
    }
    stub_names |= {
        node.target.id
        for node in tree.body
        if isinstance(node, ast.AnnAssign) and isinstance(node.target, ast.Name)
    }

    exported = {
        name
        for name in dir(lox)
        if not name.startswith("_") and name != "lox_space"
    }
    missing = exported - stub_names
    assert not missing, f"exported but missing from the stub: {sorted(missing)}"
