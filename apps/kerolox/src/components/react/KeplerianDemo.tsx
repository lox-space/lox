// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

import { Canvas } from "@react-three/fiber";
import { OrbitControls, GizmoHelper, GizmoViewport, PerspectiveCamera } from "@react-three/drei";
import { Suspense } from "react";
import { Earth, KeplerianOrbit } from "@lox-space/r3f";
import { useLox, useKeplerianControls } from "@lox-space/react";
import { Leva } from "leva";

function Scene() {
  // Ensure WASM is initialized before rendering components that use it
  useLox();

  const { semiMajorAxis, eccentricity, inclination, raan, argPeriapsis, trueAnomaly, color } =
    useKeplerianControls();

  return (
    <>
      <ambientLight intensity={2} />
      <PerspectiveCamera makeDefault position={[0, 0, 7e4]} far={1e12} />
      <OrbitControls />
      <GizmoHelper alignment="bottom-right">
        <GizmoViewport />
      </GizmoHelper>

      <Earth textureUrl="/assets/Earth-color.jpg" />
      <KeplerianOrbit
        semiMajorAxis={semiMajorAxis}
        eccentricity={eccentricity}
        inclination={inclination}
        raan={raan}
        argPeriapsis={argPeriapsis}
        trueAnomaly={trueAnomaly}
        color={color}
        name="Sat1"
      />
    </>
  );
}

export default function KeplerianDemo() {
  return (
    <>
      <Leva />
      <div className="fixed inset-0 overflow-hidden">
        <Canvas gl={{ logarithmicDepthBuffer: true }}>
          <Suspense fallback={null}>
            <Scene />
          </Suspense>
        </Canvas>
      </div>
    </>
  );
}
