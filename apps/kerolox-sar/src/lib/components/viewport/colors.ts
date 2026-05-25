// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

/**
 * Per-plane palette for satellite markers and ground tracks. Chosen to be
 * visually distinct from each other and from the AOI amber so satellites
 * never blend into an AOI polygon.
 */
const PLANE_COLORS = [
  "#7c8aff", // indigo
  "#34d399", // emerald
  "#fb7185", // rose
  "#a78bfa", // violet
  "#2dd4bf", // teal
  "#facc15", // yellow
  "#f472b6", // pink
  "#60a5fa", // sky blue
];

export function colorForPlane(plane: number): string {
  return PLANE_COLORS[plane % PLANE_COLORS.length];
}

/**
 * Parse the plane index from a satellite id of the form `pN-sM`.
 * Returns 0 for malformed ids so consumers always get a valid color.
 */
export function parsePlaneFromId(id: string): number {
  const m = id.match(/^p(\d+)-s\d+$/);
  return m ? parseInt(m[1], 10) : 0;
}
