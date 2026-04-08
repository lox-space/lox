// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

import { useEffect, useMemo } from "react";
import { useLoader } from "@react-three/fiber";
import { Origin } from "@lox-space/wasm";
import { TextureLoader } from "three";

interface EarthProps {
  textureUrl?: string;
}

export function Earth({ textureUrl }: EarthProps) {
  const earth = useMemo(() => new Origin("Earth"), []);
  useEffect(() => () => { earth.free(); }, [earth]);
  const meanRadius = earth.mean_radius() / 1000; // m to km

  return (
    <mesh>
      <sphereGeometry args={[meanRadius, 64, 64]} />
      {textureUrl ? (
        <EarthTextured url={textureUrl} />
      ) : (
        <meshStandardMaterial color="#4488ff" />
      )}
    </mesh>
  );
}

function EarthTextured({ url }: { url: string }) {
  const texture = useLoader(TextureLoader, url);
  return <meshStandardMaterial map={texture} />;
}
