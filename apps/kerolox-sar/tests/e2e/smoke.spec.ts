// SPDX-FileCopyrightText: 2026 Helge Eichhorn <git@helgeeichhorn.de>
//
// SPDX-License-Identifier: MPL-2.0

import { test, expect } from "@playwright/test";

test("scenario form drives the satellites table", async ({ page }) => {
  await page.goto("/");
  // Wait for WASM to initialise.
  await expect(page.getByTestId("satellites-table")).toBeVisible({ timeout: 15_000 });

  // Default scenario is T=24, P=3 → 24 rows.
  const rows = page.locator("[data-testid='satellites-table'] tbody tr");
  await expect(rows).toHaveCount(24);

  // Edit P to 4 (T=24 still divides cleanly): expect 4 distinct planes.
  await page.getByLabel("P — planes").fill("4");
  await expect(rows).toHaveCount(24);
  const planes = await page.locator("[data-testid='satellites-table'] tbody tr td:first-child").allTextContents();
  const distinct = new Set(planes);
  expect(distinct.size).toBe(4);
});
