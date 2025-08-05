import { test, expect } from "@playwright/test";
import { faker } from "@faker-js/faker/locale/en";
import { loginAndGoToHome, waitForLoadingOverlay } from "./shared_expects";

test("should create a folder", async ({ page }) => {
    const folderName = faker.person.fullName();

    await loginAndGoToHome(page);

    await page.getByRole("button", { name: "New folder" }).click();

    await expect(page.locator("h2", { hasText: "New folder" })).toBeVisible();

    await page.getByLabel("Name").fill(folderName);
    await page.getByRole("button", { name: "Submit" }).click();

    await expect(page.getByText("Folder created successfully")).toBeVisible();

    await page.getByRole("button", { name: "Ok" }).click();

    await expect(page.getByText(folderName)).toBeVisible()
});

test("should fail to create a folder", async ({ page }) => {
    await loginAndGoToHome(page);

    await page.getByRole("button", { name: "New folder" }).click();

    await expect(page.locator("h2", { hasText: "New folder" })).toBeVisible();

    await page.getByRole("button", { name: "Submit" }).click();

    await expect(page.getByText("Failed to create folder")).toBeVisible();
});
