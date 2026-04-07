// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

import { useControls } from "leva";

export interface KeplerianControls {
  semiMajorAxis: number;
  eccentricity: number;
  inclination: number;
  raan: number;
  argPeriapsis: number;
  trueAnomaly: number;
  color: string;
}

export function useKeplerianControls(
  defaults?: Partial<KeplerianControls>,
): KeplerianControls {
  const values = useControls("Keplerian Elements", {
    semiMajorAxis: {
      value: defaults?.semiMajorAxis ?? 24464,
      min: 7000,
      max: 100000,
      step: 100,
      label: "Semi-Major Axis [km]",
    },
    eccentricity: {
      value: defaults?.eccentricity ?? 0.7311,
      min: 0.01,
      max: 0.99,
      step: 0.01,
      label: "Eccentricity",
    },
    inclination: {
      value: defaults?.inclination ?? 7.0,
      min: 0,
      max: 180,
      step: 1,
      label: "Inclination [deg]",
    },
    raan: {
      value: defaults?.raan ?? 57.7,
      min: 0,
      max: 360,
      step: 1,
      label: "RAAN [deg]",
    },
    argPeriapsis: {
      value: defaults?.argPeriapsis ?? 178.1,
      min: 0,
      max: 360,
      step: 1,
      label: "Arg. of Periapsis [deg]",
    },
    trueAnomaly: {
      value: defaults?.trueAnomaly ?? 25.4,
      min: -180,
      max: 180,
      step: 1,
      label: "True Anomaly [deg]",
    },
    color: {
      value: defaults?.color ?? "#e92093",
      label: "Orbit Color",
    },
  });

  return values;
}
