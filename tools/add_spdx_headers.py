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
        result = subprocess.check_output(["uvx", "reuse", "lint", "-j"])
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


def add_header(file, copyright):
    """Add SPDX header with current year to the file."""
    year = str(date.today().year)
    subprocess.call(
        ["uvx", "reuse", "annotate", "-c", copyright, "-y", year, "-l", LICENSE, file]
    )


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
