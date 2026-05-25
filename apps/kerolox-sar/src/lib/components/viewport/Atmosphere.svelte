<!--
  SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
  SPDX-License-Identifier: MPL-2.0
-->
<script lang="ts">
  import { T } from "@threlte/core";
  import { AdditiveBlending, BackSide, Color, ShaderMaterial, Vector3 } from "three";

  // `radiusKm` is the solid Earth radius; the shell sits a few percent above it
  // so the glow forms a halo just outside the planet's disc. `sunDir` is the
  // unit Sun direction in the same inertial frame as this mesh.
  let {
    radiusKm,
    sunDir,
    color = "#5aa0ff",
    thickness = 0.04,
    power = 3.2,
    intensity = 1.4,
  }: {
    radiusKm: number;
    sunDir: [number, number, number];
    color?: string;
    thickness?: number;
    power?: number;
    intensity?: number;
  } = $props();

  const uniforms = {
    uSunDir: { value: new Vector3(1, 0, 0) },
    uColor: { value: new Color() },
    uPower: { value: 1 },
    uIntensity: { value: 1 },
  };

  // Keep uniforms in sync with props (Sun direction updates every frame).
  $effect(() => {
    uniforms.uSunDir.value.set(sunDir[0], sunDir[1], sunDir[2]).normalize();
    uniforms.uColor.value.set(color);
    uniforms.uPower.value = power;
    uniforms.uIntensity.value = intensity;
  });

  const vertexShader = /* glsl */ `
    varying vec3 vWorldNormal;
    varying vec3 vViewDir;
    void main() {
      vec4 worldPos = modelMatrix * vec4(position, 1.0);
      vWorldNormal = normalize(mat3(modelMatrix) * normal);
      vViewDir = normalize(cameraPosition - worldPos.xyz);
      gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
    }
  `;

  const fragmentShader = /* glsl */ `
    uniform vec3 uSunDir;
    uniform vec3 uColor;
    uniform float uPower;
    uniform float uIntensity;
    varying vec3 vWorldNormal;
    varying vec3 vViewDir;
    void main() {
      // Fresnel rim: strongest where the line of sight grazes the limb.
      float rim = pow(1.0 - abs(dot(vViewDir, vWorldNormal)), uPower);
      // Fade the glow on the hemisphere facing away from the Sun, leaving a
      // faint band so the night limb isn't a hard cut.
      float day = smoothstep(-0.4, 0.5, dot(vWorldNormal, uSunDir));
      float a = rim * mix(0.04, 1.0, day) * uIntensity;
      gl_FragColor = vec4(uColor * a, a);
    }
  `;

  const material = new ShaderMaterial({
    uniforms,
    vertexShader,
    fragmentShader,
    transparent: true,
    blending: AdditiveBlending,
    side: BackSide,
    depthWrite: false,
  });
</script>

<T.Mesh {material}>
  <T.SphereGeometry args={[radiusKm * (1 + thickness), 64, 64]} />
</T.Mesh>
