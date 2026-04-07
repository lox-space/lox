# SPDX-FileCopyrightText: 2025 Helge Eichhorn <git@helgeeichhorn.de>
#
# SPDX-License-Identifier: MPL-2.0

import argparse
from datetime import date
import json
import subprocess
import sys
import re
from pathlib import Path

LICENSE = "MPL-2.0"


def files_without_header():
    try:
        result = subprocess.check_output(
            ["uvx", "--from", "reuse[charset-normalizer]", "reuse", "lint", "-j"]
        )
    except subprocess.CalledProcessError as exc:
        result = exc.output

    return json.loads(result)["non_compliant"]["missing_licensing_info"]


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
    except Exception:
        return False


STYLE_OVERRIDES = {
    ".svelte": "html",
    ".mdx": "html",
    ".css": "c",
}

FORCE_DOT_LICENSE = {".jpg", ".jpeg", ".png", ".gif", ".svg", ".ico", ".webp"}

# Astro files need special handling: SPDX headers must be inside the frontmatter
# fences (---), not before them.
# REUSE-IgnoreStart
def astro_header(year, copyright, license):
    return (
        "---\n"
        f"// SPDX-FileCopyrightText: {year} {copyright}\n"
        "//\n"
        f"// SPDX-License-Identifier: {license}\n"
    )
# REUSE-IgnoreEnd


def add_astro_header(file, copyright):
    """Add SPDX header inside Astro frontmatter fences."""
    year = str(date.today().year)
    with open(file, "r", encoding="utf-8") as f:
        content = f.read()

    header = astro_header(year, copyright, LICENSE)

    if content.startswith("---\n"):
        # Has frontmatter — replace opening fence with fence + header
        content = header + content[4:]
    else:
        # No frontmatter — wrap in fences
        content = header + "---\n\n" + content

    with open(file, "w", encoding="utf-8") as f:
        f.write(content)


def add_header(file, copyright):
    """Add SPDX header with current year to the file."""
    year = str(date.today().year)
    suffix = Path(file).suffix.lower()

    if suffix == ".astro":
        add_astro_header(file, copyright)
        return

    cmd = [
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
    ]
    if suffix in FORCE_DOT_LICENSE:
        cmd.append("--force-dot-license")
    else:
        style = STYLE_OVERRIDES.get(suffix)
        if style:
            cmd.extend(["--style", style])
    cmd.append(file)
    subprocess.call(cmd)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(
        description="Add SPDX headers to files missing licensing information"
    )
    parser.add_argument(
        "files",
        nargs="*",
        help="Files to add headers to. If not provided, will check for non-compliant files using 'reuse lint'",
    )
    args = parser.parse_args()

    # Use provided files or find non-compliant files
    files = args.files if args.files else files_without_header()

    if not files:
        sys.exit(0)

    author = get_current_author()

    for file in files:
        # Only add header if the current author doesn't already have copyright
        if not has_author_copyright(file, author):
            add_header(file, author)
        # else: Author already has copyright in this file, skip
