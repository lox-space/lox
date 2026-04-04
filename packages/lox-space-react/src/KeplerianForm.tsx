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
