import { Planet, position, unixToJulian } from "@lox-space/core";
import { Sphere } from "@react-three/drei";
import { useFrame } from "@react-three/fiber";
import { useRef } from "react";
import { Mesh } from "three";

type Props = {
  body: Planet;
  scale?: number;
};

const CelestialBody = ({ body, scale = 1.0 }: Props) => {
  const ref = useRef<Mesh>(null!);

  useFrame(() => {
    const { date1, date2 } = unixToJulian(1e-3 * Date.now());
    if (ref.current) {
      position(ref.current.position, body, date1, date2, { toScreen: true });
    }
  });

  return (
    <>
      <Sphere ref={ref} args={[1e9, 1000, 1000]} scale={scale}>
        <meshStandardMaterial color="blue" />
      </Sphere>
    </>
  );
};

export default CelestialBody;
