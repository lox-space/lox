// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

import { defineConfig } from "astro/config";

import svelte from "@astrojs/svelte";

import tailwindcss from "@tailwindcss/vite";

import wasm from "vite-plugin-wasm";

import react from "@astrojs/react";

// https://astro.build/config
export default defineConfig({
  integrations: [svelte(), react()],

  vite: {
    plugins: [tailwindcss(), wasm()],
  },
});