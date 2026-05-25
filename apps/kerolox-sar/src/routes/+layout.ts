// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

// This app is a client-side demo — disable SSR so the WASM eager-init in
// `walker.svelte.ts` doesn't run in Node during prerender.
export const ssr = false;
export const prerender = false;
