<!--
SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>

SPDX-License-Identifier: MPL-2.0
-->

# lox-odm

Rust types and (de-)serialization for CCSDS Orbit Data Messages (ODM)
defined in [CCSDS 502.0-B-3](https://public.ccsds.org/Pubs/502x0b3e1.pdf):

- **OPM** — Orbit Parameter Message (KVN, XML)
- **OEM** — Orbit Ephemeris Message (KVN, XML)
- **OMM** — Orbit Mean Elements Message (KVN, XML, Space-Track / Celestrak JSON)

The top-level `read_*` / `write_*` functions auto-detect the wire format;
per-format modules (`kvn`, `xml`, `json`) are also exposed when the format
is known up front. Comments, user-defined keywords, and provider-specific
extras round-trip losslessly.

Part of [Lox – Oxidized Astrodynamics](https://github.com/lox-space/lox).
