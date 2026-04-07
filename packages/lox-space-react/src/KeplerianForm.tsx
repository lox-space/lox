// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

export type KeplerianElements = {
  semiMajor: number;
  eccentricity: number;
  inclination: number;
  ascendingNode: number;
  periapsisArg: number;
  trueAnomaly: number;
};

type KeplerianForm = {
  elements: KeplerianElements;
  setElements: (elements: KeplerianElements) => void;
};

export const KeplerianForm = () => {
  return <></>;
};
