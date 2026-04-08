// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

import { use } from "react";

async function init() {
  const module = await import("@lox-space/wasm");
  await module.default();
  return module;
}

let wasm: Promise<typeof import("@lox-space/wasm")> | null = null;

function getWasm() {
  if (!wasm) {
    wasm = init().catch((err) => {
      wasm = null;
      throw err;
    });
  }
  return wasm;
}

export function useLox() {
  return use(getWasm());
}
