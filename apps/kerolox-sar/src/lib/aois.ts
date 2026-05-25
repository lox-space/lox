// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

export interface AoiPolygon {
  id: string;
  name: string;
  /** Exterior ring as parallel [lon, lat] arrays. */
  exteriorLonLat: Array<[number, number]>;
}

type GeoJsonFeatureCollection = {
  features: Array<{
    properties: { id?: string; name?: string };
    geometry: { type: string; coordinates: Array<Array<[number, number]>> };
  }>;
};

const AOI_FILES = [
  { id: "hormuz", path: "/aois/hormuz.geojson" },
  { id: "black_sea", path: "/aois/black_sea.geojson" },
];

let cache: Map<string, AoiPolygon> | null = null;

/** Fetch all AOI polygons. Idempotent — caches after the first call. */
export async function loadAois(): Promise<Map<string, AoiPolygon>> {
  if (cache) return cache;
  const out = new Map<string, AoiPolygon>();
  for (const { id, path } of AOI_FILES) {
    const res = await fetch(path);
    if (!res.ok) throw new Error(`failed to load ${path}: ${res.status}`);
    const data = (await res.json()) as GeoJsonFeatureCollection;
    const feature = data.features[0];
    if (!feature) throw new Error(`${path}: no features`);
    out.set(id, {
      id,
      name: feature.properties.name ?? id,
      exteriorLonLat: feature.geometry.coordinates[0],
    });
  }
  cache = out;
  return out;
}
