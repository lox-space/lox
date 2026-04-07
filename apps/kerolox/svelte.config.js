// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

import { vitePreprocess } from "@astrojs/svelte";

export default {
  preprocess: vitePreprocess(),
  compilerOptions: {
    experimental: {
      async: true,
    },
  },
};
