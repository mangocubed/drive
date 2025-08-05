import { test, expect } from "@playwright/test";

test("should have a heading text", async ({ page }) => {
    await page.goto("/");

    await expect(page.locator(".loading-overlay")).toHaveClass(/is-done/);
    await expect(page.locator("h1", { hasText: "Home" })).toBeVisible();
});
