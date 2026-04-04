import { OrbitControls, PerspectiveCamera, Stats } from "@react-three/drei";
import { Canvas } from "@react-three/fiber";
import CelestialBody from "./CelestialBody";
import StarMap from "./StarMap";

const Planetarium = () => {
  return (
    <Canvas
      onCreated={({ gl }) => gl.setClearColor("#000000")}
      gl={{ logarithmicDepthBuffer: true }}
    >
      <Stats showPanel={0} className="stats" />
      <StarMap type="grid" />
      <ambientLight />
      <pointLight position={[10, 10, 10]} />
      <CelestialBody body="mercury" />
      <CelestialBody body="venus" />
      <CelestialBody body="earthMoonBarycenter" />
      <CelestialBody body="mars" />
      <CelestialBody body="jupiter" />
      <CelestialBody body="saturn" />
      <CelestialBody body="uranus" />
      <CelestialBody body="neptune" />
      <OrbitControls />
      <PerspectiveCamera makeDefault position={[0, 0, 1e8]} far={1e24} />
    </Canvas>
  );
};

export default Planetarium;
