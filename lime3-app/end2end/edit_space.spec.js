import { test, expect } from "@playwright/test";
import { faker } from "@faker-js/faker/locale/en";
import { loginAndGoToHome, waitForLoadingOverlay } from "./shared_expects";

test("should be a link to storage page", async ({ page }) => {
    await loginAndGoToHome(page);

    await waitForLoadingOverlay(page);

    page.getByRole("link", { name: "Storage" }).click();

    await expect(page).toHaveURL("/storage");
    await expect(page.locator("h1", { hasText: "Storage" })).toBeVisible();
});

test("should open modal to edit space quota", async ({ page }) => {
    await loginAndGoToHome(page);

    await waitForLoadingOverlay(page);

    page.getByRole("link", { name: "Storage" }).click();

    page.getByRole("button", { name: "Edit space" }).click();

    await expect(page.locator("h2", { hasText: "Edit space" })).toBeVisible();
});
