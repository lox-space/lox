// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

import { use } from "react";

async function init() {
  const module = await import("@lox-space/wasm");
  await module.default();
  return module;
}

const wasm = init();

export function useLox() {
  return use(wasm);
}
