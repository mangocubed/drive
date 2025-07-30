import { test, expect } from "@playwright/test";
import { faker } from "@faker-js/faker/locale/en";
import { execSync } from "child_process";

test("should be a link to login page", async ({ page }) => {
    await page.goto("/");

    await expect(page.locator(".loading-overlay")).toHaveClass(/is-done/);

    page.getByRole("link", { name: "Login" }).click();

    await expect(page).toHaveURL("/login");
    await expect(page.locator("h1", { hasText: "Login" })).toBeVisible();
});

test("should login and logout a user", async ({ page }) => {
    const username = faker.internet.username().substring(0, 16);
    const email = `${username}@example.com`;
    const password = faker.internet.password();
    const result = execSync(
        `cargo run --bin cli --features cli create-user \
                --username '${username}' --email '${email}' --password '${password}' \
                --full-name 'Test User' --birthdate '1990-01-01' --country 'VE'`,
    );

    expect(result.toString()).toContain("User created successfully.");

    await page.goto("/login");

    await expect(page.locator(".loading-overlay")).toHaveClass(/is-done/);
    await expect(page.locator("h1", { hasText: "Login" })).toBeVisible();

    await page.getByLabel("Username or email").fill(username);
    await page.getByLabel("Password").fill(password);

    await page.getByRole("button", { name: "Submit" }).click();

    await expect(page.getByText("User authenticated successfully")).toBeVisible();

    await page.getByRole("button", { name: "Ok" }).click();

    await expect(page).toHaveURL("/");
    await expect(page.locator("h1", { hasText: "Home" })).toBeVisible();

    await page.getByRole("button", { name: `@${username}` }).click();
    await page.locator("a", { hasText: "Logout" }).click();

    await expect(page.getByText("Are you sure you want to logout?")).toBeVisible();

    await page.getByRole("button", { name: "Accept" }).click();

    await expect(page.locator("h1", { hasText: "Login" })).toBeVisible();
});

test("should fail to login a user", async ({ page }) => {
    await page.goto("/login");

    await expect(page.locator(".loading-overlay")).toHaveClass(/is-done/);
    await expect(page.locator("h1", { hasText: "Login" })).toBeVisible();

    await page.getByLabel("Username or email").fill(faker.internet.username());
    await page.getByLabel("Password").fill(faker.internet.password());

    await page.getByRole("button", { name: "Submit" }).click();

    await expect(page.getByText("Failed to authenticate user")).toBeVisible();
});
