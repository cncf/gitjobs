import { test, expect } from "@playwright/test";
import {
  clickUserMenuItem,
  loginWithCredentials,
  openLoginPage,
  openSignUpPage,
  openUserMenu,
} from "../shared/utils";
import { navigateHomeWithRetries } from "../shared/helpers";

test.describe("GitJobs - Auth and Nav", () => {
  test.beforeEach(async ({ page }) => {
    await navigateHomeWithRetries(page);
  });

  test("should navigate to the sign-up page", async ({ page }) => {
    await openSignUpPage(page);
    await expect(page).toHaveURL(/\/sign-up(?:\?.*)?$/);
  });

  test("should block sign-up when passwords do not match", async ({ page }) => {
    await openSignUpPage(page);

    await page.locator("#name").fill("Test User");
    await page.locator("#email").fill("test-signup@example.com");
    await page.locator("#username").fill("test-signup");
    await page.locator("#password").fill("test1234");
    await page.locator("#password_confirmation").fill("test5678");
    await page.getByRole("button", { name: "Submit" }).click();

    await expect(page).toHaveURL(/\/sign-up(?:\?.*)?$/);
    await expect
      .poll(async () =>
        page
          .locator("#password_confirmation")
          .evaluate((element) => element.validationMessage),
      )
      .toBe("Passwords do not match");
  });

  test("should block sign-up when required name contains only spaces", async ({
    page,
  }) => {
    await openSignUpPage(page);

    await page.locator("#name").fill("   ");
    await page.locator("#email").fill("test-spaces@example.com");
    await page.locator("#username").fill("test-spaces");
    await page.locator("#password").fill("test1234");
    await page.locator("#password_confirmation").fill("test1234");
    await page.getByRole("button", { name: "Submit" }).click();

    await expect(page).toHaveURL(/\/sign-up(?:\?.*)?$/);
    await expect
      .poll(async () =>
        page.locator("#name").evaluate((element) => element.validationMessage),
      )
      .toBe("Value cannot be empty");
  });

  test("should log in a user", async ({ page }) => {
    await loginWithCredentials(page, "test", "test1234");
    await expect(page).toHaveURL(/\/(?:\?.*)?$/);
  });

  test("should log out a user", async ({ page }) => {
    await loginWithCredentials(page, "test", "test1234");

    await expect(page).toHaveURL(/\/(?:\?.*)?$/);
    await openUserMenu(page);
    await clickUserMenuItem(page, "Log out");
    await page.waitForURL(/\/log-in(?:\?.*)?$/);
  });

  test("should close user menu on Escape and restore focus", async ({
    page,
  }) => {
    const userButton = page.locator("#user-dropdown-button");
    const dropdown = page.locator("#dropdown-user");

    await openUserMenu(page);
    await expect(dropdown).toBeVisible();
    await expect(userButton).toHaveAttribute("aria-expanded", "true");
    await expect(dropdown).toHaveAttribute("aria-hidden", "false");

    await page.keyboard.press("Escape");

    await expect(dropdown).toBeHidden();
    await expect(userButton).toBeFocused();
    await expect(userButton).toHaveAttribute("aria-expanded", "false");
    await expect(dropdown).toHaveAttribute("aria-hidden", "true");
  });

  test("should toggle user menu aria state on click", async ({ page }) => {
    const userButton = page.locator("#user-dropdown-button");
    const dropdown = page.locator("#dropdown-user");

    await openUserMenu(page);
    await expect(dropdown).toBeVisible();
    await expect(userButton).toHaveAttribute("aria-expanded", "true");
    await expect(dropdown).toHaveAttribute("aria-hidden", "false");

    await userButton.click();
    await expect(dropdown).toBeHidden();
    await expect(userButton).toHaveAttribute("aria-expanded", "false");
    await expect(dropdown).toHaveAttribute("aria-hidden", "true");
  });

  test("should keep user menu toggle stable after HTMX navigation", async ({
    page,
  }) => {
    const headerNavigation = page.locator("#header nav").first();

    await headerNavigation
      .getByRole("link", { name: "About", exact: true })
      .click();
    await expect(page).toHaveURL(/\/about(?:\?.*)?$/);
    await headerNavigation
      .getByRole("link", { name: "Jobs", exact: true })
      .click();
    await expect(page).toHaveURL(/\/(?:\?.*)?$/);

    const userButton = page.locator("#user-dropdown-button");
    const dropdown = page.locator("#dropdown-user");

    await userButton.click();
    await expect(dropdown).toBeVisible();
    await expect(userButton).toHaveAttribute("aria-expanded", "true");

    await userButton.click();
    await expect(dropdown).toBeHidden();
    await expect(userButton).toHaveAttribute("aria-expanded", "false");

    await userButton.click();
    await expect(dropdown).toBeVisible();
    await expect(userButton).toHaveAttribute("aria-expanded", "true");
  });

  test("invalid credentials stay on log in page", async ({ page }) => {
    await openLoginPage(page);

    await page.locator("#username").fill("test");
    await page.locator("#password").fill("wrong");
    await page.getByRole("button", { name: "Submit" }).click();

    await expect(page).toHaveURL(/\/log-in(?:\?.*)?$/);
  });
});
