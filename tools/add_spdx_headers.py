# SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
#
# SPDX-License-Identifier: MPL-2.0

"""Ensure the current Git author has an SPDX copyright line in new or modified files.

Files covered by an annotation in REUSE.toml (machine-generated files, data, etc.) are
skipped. An existing copyright line for the author is never touched, so the year records
the author's first contribution to the file.
"""

import argparse
import re
import subprocess
import sys
import tomllib
from datetime import date
from pathlib import Path

LICENSE = "MPL-2.0"
ROOT = Path(
    subprocess.check_output(["git", "rev-parse", "--show-toplevel"]).decode().strip()
)


def changed_files():
    """Tracked files that are added or modified relative to HEAD, staged or not."""
    output = subprocess.check_output(
        ["git", "diff", "HEAD", "--name-only", "--diff-filter=AM", "-z"],
        cwd=ROOT,
    )
    return [f for f in output.decode("utf-8").split("\0") if f]


def reuse_glob_to_regex(pattern):
    """Translate a REUSE.toml path glob: `**` matches across `/`, `*` does not."""
    regex = ""
    i = 0
    while i < len(pattern):
        if pattern.startswith("\\*", i):
            regex += re.escape("*")
            i += 2
        elif pattern.startswith("**/", i):
            # Also matches files at the top level, e.g. `**/Cargo.lock` -> `Cargo.lock`
            regex += "(?:.*/)?"
            i += 3
        elif pattern.startswith("**", i):
            regex += ".*"
            i += 2
        elif pattern[i] == "*":
            regex += "[^/]*"
            i += 1
        else:
            regex += re.escape(pattern[i])
            i += 1
    return re.compile(regex)


def reuse_toml_patterns():
    """Path patterns of all annotations in REUSE.toml."""
    reuse_toml = ROOT / "REUSE.toml"
    if not reuse_toml.exists():
        return []
    with open(reuse_toml, "rb") as f:
        annotations = tomllib.load(f).get("annotations", [])
    patterns = []
    for annotation in annotations:
        paths = annotation["path"]
        if isinstance(paths, str):
            paths = [paths]
        patterns.extend(reuse_glob_to_regex(p) for p in paths)
    return patterns


def is_covered_by_reuse_toml(file, patterns):
    relative = Path(file).resolve().relative_to(ROOT).as_posix()
    return any(p.fullmatch(relative) for p in patterns)


def get_current_author():
    author = (
        subprocess.check_output(["git", "config", "user.name"]).decode("utf-8").strip()
    )
    email = (
        subprocess.check_output(["git", "config", "user.email"]).decode("utf-8").strip()
    )
    return f"{author} <{email}>"


def has_author_copyright(file, author):
    """Check if the file already contains a copyright line for the given author."""
    try:
        with open(file, "r", encoding="utf-8") as f:
            content = f.read()
            # Look for SPDX-FileCopyrightText lines containing the author
            # Match patterns like: SPDX-FileCopyrightText: YEAR Author <email>
            # or: SPDX-FileCopyrightText: Author <email>
            pattern = rf"SPDX-FileCopyrightText:.*{re.escape(author)}"
            return bool(re.search(pattern, content))
    except (OSError, UnicodeDecodeError):
        return False


def add_headers(files, copyright):
    """Add an SPDX copyright line with the current year to the files."""
    year = str(date.today().year)
    return subprocess.call(
        [
            "uvx",
            "--from",
            "reuse[charset-normalizer]",
            "reuse",
            "annotate",
            "-c",
            copyright,
            "-y",
            year,
            "-l",
            LICENSE,
            *files,
        ]
    )


if __name__ == "__main__":
    parser = argparse.ArgumentParser(
        description="Add the current author's SPDX header to new or modified files"
    )
    parser.add_argument(
        "files",
        nargs="*",
        help="Files to check. Defaults to tracked files added or modified since HEAD.",
    )
    args = parser.parse_args()

    files = args.files if args.files else [ROOT / f for f in changed_files()]
    patterns = reuse_toml_patterns()
    author = get_current_author()

    missing = [
        str(f)
        for f in files
        if not is_covered_by_reuse_toml(f, patterns)
        and not has_author_copyright(f, author)
    ]

    if missing:
        sys.exit(add_headers(missing, author))
