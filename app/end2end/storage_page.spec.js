import { test, expect } from "@playwright/test";
import { waitForSplash, testOptions } from "./shared_expects";

test.use(testOptions());

test("should be a link to storage page", async ({ page }) => {
    await page.goto("/");

    await waitForSplash(page);

    page.getByRole("link", { name: "Storage" }).click();

    await expect(page).toHaveURL("/storage");
    await expect(page.locator("h1", { hasText: "Storage" })).toBeVisible();
});

test("should open modal to select a plan", async ({ page }) => {
    await page.goto("/");

    await waitForSplash(page);

    page.getByRole("link", { name: "Storage" }).click();

    page.getByRole("button", { name: "Get more space" }).click();

    await expect(page.locator("h2", { hasText: "Select plan" })).toBeVisible();
});
