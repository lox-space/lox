import { Canvas } from "@react-three/fiber";
import { Suspense } from "react";
import Scene from "./Scene.tsx";

const KeplerianWidget = () => {
  return (
    <div className="fixed inset-0 overflow-hidden">
      <Canvas
        onCreated={({ gl }) => gl.setClearColor("#000000")}
        gl={{ logarithmicDepthBuffer: true }}
      >
        <Suspense fallback={null}>
          <Scene />
        </Suspense>
      </Canvas>
    </div>
  );
};

export default KeplerianWidget;
