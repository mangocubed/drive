import { test, expect } from "@playwright/test";
import { faker } from "@faker-js/faker/locale/en";

test("should register a new user", async ({ page }) => {
    await page.goto("/");

    await expect(page.locator(".loading-overlay")).toHaveClass(/is-done/);

    page.getByRole("link", { name: "Register" }).click();

    await expect(page.locator("h1", { hasText: "Register" })).toBeVisible();

    await page.getByLabel("Username").fill(faker.internet.username().substring(0, 16));
    await page.getByLabel("Email").fill(faker.internet.email());
    await page.getByLabel("Password").fill(faker.internet.password());
    await page.getByLabel("Full name").fill(faker.person.fullName());
    await page.getByLabel("Birthdate").fill(faker.date.birthdate().toISOString().split("T")[0]);
    await page.getByLabel("Country").selectOption(faker.location.countryCode());

    await page.getByRole("button", { name: "Submit" }).click();

    await expect(page.getByText("User created successfully")).toBeVisible();
});

test("should fail to register a new user", async ({ page }) => {
    await page.goto("/");

    await expect(page.locator(".loading-overlay")).toHaveClass(/is-done/);

    page.getByRole("link", { name: "Register" }).click();

    await expect(page.locator("h1", { hasText: "Register" })).toBeVisible();

    await page.getByRole("button", { name: "Submit" }).click();

    await expect(page.getByText("Failed to create user")).toBeVisible();
});
