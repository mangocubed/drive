import { test, expect } from "@playwright/test";
import { faker } from "@faker-js/faker/locale/en";
import path from "path";
import { loginAndGoToHome, waitForLoadingOverlay } from "./shared_expects";

test("should upload files", async ({ page }) => {
    await loginAndGoToHome(page);

    const fileChooserPromise = page.waitForEvent("filechooser");

    await page.locator("label", { hasText: "Upload files" }).click();

    const fileChooser = await fileChooserPromise;

    await fileChooser.setFiles(path.join(__dirname, "../assets/favicon.png"));

    await expect(page.locator("h2", { hasText: "Uploading files" })).toBeVisible();
    await expect(page.getByRole("button", { name: "Close" })).toBeEnabled();

    await page.getByRole("button", { name: "Close" }).click();

    await expect(page.getByRole("link", { name: "favicon.png" })).toBeVisible();
});
