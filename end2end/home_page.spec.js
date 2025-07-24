import { test, expect } from "@playwright/test";

test("has a heading text", async ({ page }) => {
    await page.goto("/");

    await expect(page.locator("h1", { hasText: "Home" })).toBeVisible();
});
