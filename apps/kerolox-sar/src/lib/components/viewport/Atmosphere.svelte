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
  // `dayColor` tints the sunlit limb; `sunsetColor` warms the terminator band
  // (Rayleigh reddening). `mie` controls the forward-scatter halo toward the Sun.
  let {
    radiusKm,
    sunDir,
    dayColor = "#7fb2ff",
    sunsetColor = "#ff7038",
    thickness = 0.04,
    power = 3.0,
    intensity = 1.0,
    mie = 0.5,
  }: {
    radiusKm: number;
    sunDir: [number, number, number];
    dayColor?: string;
    sunsetColor?: string;
    thickness?: number;
    power?: number;
    intensity?: number;
    mie?: number;
  } = $props();

  const uniforms = {
    uSunDir: { value: new Vector3(1, 0, 0) },
    uDayColor: { value: new Color() },
    uSunsetColor: { value: new Color() },
    uPower: { value: 1 },
    uIntensity: { value: 1 },
    uMie: { value: 0.5 },
  };

  // Keep uniforms in sync with props (Sun direction updates every frame).
  $effect(() => {
    uniforms.uSunDir.value.set(sunDir[0], sunDir[1], sunDir[2]).normalize();
    uniforms.uDayColor.value.set(dayColor);
    uniforms.uSunsetColor.value.set(sunsetColor);
    uniforms.uPower.value = power;
    uniforms.uIntensity.value = intensity;
    uniforms.uMie.value = mie;
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
    uniform vec3 uDayColor;
    uniform vec3 uSunsetColor;
    uniform float uPower;
    uniform float uIntensity;
    uniform float uMie;
    varying vec3 vWorldNormal;
    varying vec3 vViewDir;

    const float PI = 3.14159265359;
    const float MIE_G = 0.76;

    // Rayleigh phase: mild forward/back symmetry, brightens near the Sun.
    float rayleighPhase(float mu) {
      return (3.0 / (16.0 * PI)) * (1.0 + mu * mu);
    }
    // Henyey-Greenstein Mie phase: sharp forward lobe (the bright sun-side halo).
    float miePhase(float mu) {
      float gg = MIE_G * MIE_G;
      float num = 3.0 * (1.0 - gg) * (1.0 + mu * mu);
      float den = 8.0 * PI * (2.0 + gg) * pow(max(1.0 + gg - 2.0 * MIE_G * mu, 1e-4), 1.5);
      return num / den;
    }

    void main() {
      vec3 N = normalize(vWorldNormal);
      vec3 V = normalize(vViewDir);
      vec3 S = normalize(uSunDir);

      // Fresnel rim: strongest where the line of sight grazes the limb.
      float rim = pow(1.0 - abs(dot(V, N)), uPower);

      // How sunlit this patch of shell is. Drives the day/night fade and the
      // position of the terminator (sunset) band.
      float muSun = dot(N, S);
      float day = smoothstep(-0.35, 0.30, muSun);

      // Scattering angle between sunlight and the view ray feeds the phase
      // functions: Rayleigh gives a gentle blue wash, Mie a sun-side halo.
      float mu = dot(V, S);
      float bright = 0.7 + 2.0 * rayleighPhase(mu);
      float halo = miePhase(mu) * uMie;

      // Warm reddening concentrated just inside the lit side (longer optical
      // path near the terminator → Rayleigh reddening of the transmitted light).
      float warmth = (1.0 - smoothstep(0.0, 0.45, muSun)) * day;
      vec3 col = mix(uDayColor * bright, uSunsetColor, warmth);
      col += uDayColor * halo;

      float a = rim * mix(0.03, 1.0, day) * uIntensity;
      gl_FragColor = vec4(col * a, a);
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
