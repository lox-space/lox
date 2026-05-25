// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

// Re-export the generated proto types. `buf generate` writes to `src/gen/`.
// If this import fails, run `pnpm --filter @kerolox/proto-ts generate` first.
export * from "./gen/kerolox/v1/kerolox_pb.js";
