// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

import { defineConfig } from "tsdown";

export default defineConfig({
  platform: "browser",
  deps: {
    neverBundle: [/^@lox-space\//, "react", "react-dom", "three"],
  },
});
