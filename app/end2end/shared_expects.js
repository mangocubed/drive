import { execSync } from "child_process";
import { expect, test } from "@playwright/test";

export async function waitForSplash(page) {
    await expect(page.locator(".splash")).toHaveClass(/splash-hidden/);
}

export function testOptions() {
    const result = execSync("cargo run --package drive-cli --features test-utils create-test-session");
    const session = JSON.parse(result);

    return {
        storageState: {
            origins: [
                { origin: "http://localhost:8090", localStorage: [{ name: "_session_token", value: session.token }] },
            ],
        },
    };
}
