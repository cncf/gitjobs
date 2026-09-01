import { expect, test } from "@playwright/test";
import {
  clickUserMenuItem,
  loginWithCredentials,
  openLoginPage,
  openSignUpPage,
  openUserMenu,
} from "../../shared/auth.js";
import { navigateHomeWithRetries } from "../../shared/navigation.js";

test.describe("GitJobs - Authentication", () => {
  test.beforeEach(async ({ page }) => {
    // Load the public home page before each authentication assertion.
    await navigateHomeWithRetries(page);
  });

  test("navigates to the sign-up page", async ({ page }) => {
    // Open sign-up from the public user menu.
    await openSignUpPage(page);

    // Verify the browser reaches the sign-up page.
    await expect(page).toHaveURL(/\/sign-up(?:\?.*)?$/);
  });

  test("blocks sign-up when passwords do not match", async ({ page }) => {
    // Load the public sign-up form.
    await openSignUpPage(page);

    // Submit otherwise valid fields with mismatched passwords.
    await page.locator("#name").fill("Test User");
    await page.locator("#email").fill("test-signup@example.com");
    await page.locator("#username").fill("test-signup");
    await page.locator("#password").fill("test1234");
    await page.locator("#password_confirmation").fill("test5678");
    await page.getByRole("button", { name: "Submit" }).click();

    // Verify client validation keeps the user on sign-up with an explicit error.
    await expect(page).toHaveURL(/\/sign-up(?:\?.*)?$/);
    await expect
      .poll(async () =>
        page.locator("#password_confirmation").evaluate((element) => element.validationMessage),
      )
      .toBe("Passwords do not match");
  });

  test("blocks sign-up when required name contains only spaces", async ({ page }) => {
    // Load the public sign-up form.
    await openSignUpPage(page);

    // Submit otherwise valid fields with a whitespace-only required name.
    await page.locator("#name").fill("   ");
    await page.locator("#email").fill("test-spaces@example.com");
    await page.locator("#username").fill("test-spaces");
    await page.locator("#password").fill("test1234");
    await page.locator("#password_confirmation").fill("test1234");
    await page.getByRole("button", { name: "Submit" }).click();

    // Verify client validation keeps the user on sign-up with an explicit error.
    await expect(page).toHaveURL(/\/sign-up(?:\?.*)?$/);
    await expect
      .poll(async () => page.locator("#name").evaluate((element) => element.validationMessage))
      .toBe("Value cannot be empty");
  });

  test("logs in a user", async ({ page }) => {
    // Log in with the seeded default user.
    await loginWithCredentials(page, "test", "test1234");

    // Verify authentication returns the user to the job board.
    await expect(page).toHaveURL(/\/(?:\?.*)?$/);
  });

  test("logs out a user", async ({ page }) => {
    // Log in with the seeded default user.
    await loginWithCredentials(page, "test", "test1234");

    // Open the authenticated menu and request logout.
    await expect(page).toHaveURL(/\/(?:\?.*)?$/);
    await openUserMenu(page);
    await clickUserMenuItem(page, "Log out");

    // Verify logout returns the user to the login page.
    await page.waitForURL(/\/log-in(?:\?.*)?$/);
  });

  test("invalid credentials stay on log in page", async ({ page }) => {
    // Load the public login form.
    await openLoginPage(page);

    // Submit an invalid password for the seeded user.
    await page.locator("#username").fill("test");
    await page.locator("#password").fill("wrong");
    await page.getByRole("button", { name: "Submit" }).click();

    // Verify authentication failure keeps the user on the login page.
    await expect(page).toHaveURL(/\/log-in(?:\?.*)?$/);
  });
});
