// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

import { defineConfig } from "tsdown";
import type { Plugin } from "rolldown";

const assetExternalPlugin: Plugin = {
  name: "asset-external",
  resolveId(source) {
    if (/\.(jpg|hdr)$/.test(source)) {
      return { id: source, external: true };
    }
  },
};

export default defineConfig({
  deps: {
    neverBundle: [
      /^@lox-space\//,
      "react",
      "react-dom",
      "three",
      "@react-three/fiber",
      "@react-three/drei",
      "zustand",
    ],
  },
  plugins: [assetExternalPlugin],
});
