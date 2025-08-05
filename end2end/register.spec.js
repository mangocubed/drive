import { test, expect } from "@playwright/test";
import { faker } from "@faker-js/faker/locale/en";
import { waitForLoadingOverlay } from "./shared_expects";

test("should be a link to register page", async ({ page }) => {
    await page.goto("/");

    await waitForLoadingOverlay(page);

    page.getByRole("link", { name: "Login" }).click();

    await expect(page).toHaveURL("/login");
    await expect(page.locator("h1", { hasText: "Login" })).toBeVisible();

    page.getByRole("link", { name: "I don't have an account" }).click();

    await expect(page).toHaveURL("/register");
    await expect(page.locator("h1", { hasText: "Register" })).toBeVisible();
});

test("should register a new user", async ({ page }) => {
    const username = faker.internet.username().substring(0, 16);

    await page.goto("/register");

    await waitForLoadingOverlay(page);

    await expect(page.locator("h1", { hasText: "Register" })).toBeVisible();

    await page.getByLabel("Username").fill(username);
    await page.getByLabel("Email").fill(faker.internet.email());
    await page.getByLabel("Password").fill(faker.internet.password());
    await page.getByLabel("Full name").fill(faker.person.fullName());
    await page.getByLabel("Birthdate").fill(faker.date.birthdate().toISOString().split("T")[0]);
    await page.getByLabel("Country").selectOption(faker.location.countryCode());

    await page.getByRole("button", { name: "Submit" }).click();

    await expect(page.getByText("User created successfully")).toBeVisible();

    await page.getByRole("button", { name: "Ok" }).click();

    await expect(page).toHaveURL("/");
    await expect(page.locator("h1", { hasText: "Home" })).toBeVisible();
    await expect(page.getByRole("button", { name: `@${username}` })).toBeVisible();
});

test("should fail to register a new user", async ({ page }) => {
    await page.goto("/register");

    await expect(page.locator(".loading-overlay")).toHaveClass(/is-done/);
    await expect(page.locator("h1", { hasText: "Register" })).toBeVisible();

    await page.getByRole("button", { name: "Submit" }).click();

    await expect(page.getByText("Failed to create user")).toBeVisible();
});
