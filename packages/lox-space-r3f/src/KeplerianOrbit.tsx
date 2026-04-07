// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

import { useMemo } from "react";
import { Billboard, Line, Text } from "@react-three/drei";
import { Keplerian as KeplerianWasm, Origin } from "@lox-space/wasm";
import { Vector3 } from "three";

const DEG_TO_RAD = Math.PI / 180;
const M_TO_KM = 1e-3;

interface KeplerianOrbitProps {
  semiMajorAxis: number;
  eccentricity: number;
  inclination: number;
  raan: number;
  argPeriapsis: number;
  trueAnomaly: number;
  origin?: string;
  color?: string;
  name?: string;
}

export function KeplerianOrbit({
  semiMajorAxis,
  eccentricity,
  inclination,
  raan,
  argPeriapsis,
  trueAnomaly,
  origin: originName = "Earth",
  color = "#e92093",
  name,
}: KeplerianOrbitProps) {
  const wasmOrigin = useMemo(() => new Origin(originName), [originName]);

  const orbit = useMemo(
    () =>
      new KeplerianWasm(
        semiMajorAxis * 1000,
        eccentricity,
        inclination * DEG_TO_RAD,
        raan * DEG_TO_RAD,
        argPeriapsis * DEG_TO_RAD,
        trueAnomaly * DEG_TO_RAD,
        wasmOrigin,
      ),
    [semiMajorAxis, eccentricity, inclination, raan, argPeriapsis, trueAnomaly, wasmOrigin],
  );

  const points = useMemo(() => {
    const buffer = orbit.trace(360).to_threejs_buffer();
    const pts: Vector3[] = [];
    for (let i = 0; i < buffer.length; i += 3) {
      pts.push(
        new Vector3(
          buffer[i] * M_TO_KM,
          buffer[i + 1] * M_TO_KM,
          buffer[i + 2] * M_TO_KM,
        ),
      );
    }
    return pts;
  }, [orbit]);

  const position = useMemo((): [number, number, number] => {
    const pos = orbit.to_cartesian().to_threejs();
    return [pos[0] * M_TO_KM, pos[1] * M_TO_KM, pos[2] * M_TO_KM];
  }, [orbit]);

  return (
    <>
      <Line points={points} color={color} lineWidth={2} />
      {name && (
        <Billboard position={position}>
          <Text fontSize={500} color={color}>
            {name}
          </Text>
          <mesh>
            <circleGeometry args={[100, 32]} />
            <meshBasicMaterial color={color} />
          </mesh>
        </Billboard>
      )}
    </>
  );
}
