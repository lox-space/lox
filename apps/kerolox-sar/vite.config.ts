// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

import { sveltekit } from "@sveltejs/kit/vite";
import tailwindcss from "@tailwindcss/vite";
import wasm from "vite-plugin-wasm";
import { searchForWorkspaceRoot } from "vite";
import { defineConfig } from "vitest/config";

export default defineConfig({
  plugins: [tailwindcss(), wasm(), sveltekit()],
  server: {
    fs: {
      allow: [
        searchForWorkspaceRoot(process.cwd()),
        "../../packages/lox-space-wasm",
      ],
    },
  },
  test: {
    environment: "jsdom",
    include: ["src/**/*.{test,spec}.{js,ts}"],
  },
});
