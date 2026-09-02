import { expect, test } from "@playwright/test";
import { clickUserMenuItem, loginWithCredentials, openUserMenu } from "../../shared/auth.js";
import { navigateHomeWithRetries } from "../../shared/navigation.js";

test.describe("GitJobs - Navigation", () => {
  test.beforeEach(async ({ page }) => {
    // Load the public home page before each navigation assertion.
    await navigateHomeWithRetries(page);
  });

  test("navigates to jobs from the mobile user menu @mobile", async ({ page }) => {
    // Log in using the configured mobile browser project.
    await loginWithCredentials(page, "test", "test1234");

    // Navigate away from the job board through the mobile user menu.
    await openUserMenu(page);
    await clickUserMenuItem(page, "About");
    await expect(page).toHaveURL(/\/about(?:\?.*)?$/);

    // Find the mobile Jobs destination and verify its contract.
    await openUserMenu(page);
    const jobsMenuItem = page.locator("#dropdown-user").getByRole("menuitem", {
      name: "Jobs",
      exact: true,
    });
    await expect(jobsMenuItem).toBeVisible();
    await expect(jobsMenuItem).toHaveAttribute("href", "/");

    // Follow the Jobs destination.
    await jobsMenuItem.click();

    // Verify the user returns to the job board.
    await expect(page).toHaveURL(/\/(?:\?.*)?$/);
  });

  test("closes user menu on Escape and restores focus", async ({ page }) => {
    // Find the public user menu and its trigger.
    const userButton = page.locator("#user-dropdown-button");
    const dropdown = page.locator("#dropdown-user");

    // Open the user menu and verify its accessible state.
    await openUserMenu(page);
    await expect(dropdown).toBeVisible();
    await expect(userButton).toHaveAttribute("aria-expanded", "true");
    await expect(dropdown).toHaveAttribute("aria-hidden", "false");

    // Close the user menu with Escape.
    await page.keyboard.press("Escape");

    // Verify focus returns to the trigger and the menu is closed.
    await expect(dropdown).toBeHidden();
    await expect(userButton).toBeFocused();
    await expect(userButton).toHaveAttribute("aria-expanded", "false");
    await expect(dropdown).toHaveAttribute("aria-hidden", "true");
  });

  test("toggles user menu aria state on click", async ({ page }) => {
    // Find the public user menu and its trigger.
    const userButton = page.locator("#user-dropdown-button");
    const dropdown = page.locator("#dropdown-user");

    // Open the user menu and verify its accessible state.
    await openUserMenu(page);
    await expect(dropdown).toBeVisible();
    await expect(userButton).toHaveAttribute("aria-expanded", "true");
    await expect(dropdown).toHaveAttribute("aria-hidden", "false");

    // Close the user menu through its trigger.
    await userButton.click();

    // Verify the trigger and menu expose the closed state.
    await expect(dropdown).toBeHidden();
    await expect(userButton).toHaveAttribute("aria-expanded", "false");
    await expect(dropdown).toHaveAttribute("aria-hidden", "true");
  });

  test("keeps user menu toggle stable after HTMX navigation", async ({ page }) => {
    // Find the desktop header navigation.
    const headerNavigation = page.locator("#header nav").first();

    // Navigate away from and back to the job board through HTMX links.
    await headerNavigation.getByRole("link", { name: "About", exact: true }).click();
    await expect(page).toHaveURL(/\/about(?:\?.*)?$/);
    await headerNavigation.getByRole("link", { name: "Jobs", exact: true }).click();
    await expect(page).toHaveURL(/\/(?:\?.*)?$/);

    // Find the swapped user menu and its trigger.
    const userButton = page.locator("#user-dropdown-button");
    const dropdown = page.locator("#dropdown-user");

    // Open the menu after navigation.
    await userButton.click();
    await expect(dropdown).toBeVisible();
    await expect(userButton).toHaveAttribute("aria-expanded", "true");

    // Close the menu and verify its collapsed state.
    await userButton.click();
    await expect(dropdown).toBeHidden();
    await expect(userButton).toHaveAttribute("aria-expanded", "false");

    // Reopen the menu to prove the trigger was bound only once.
    await userButton.click();
    await expect(dropdown).toBeVisible();
    await expect(userButton).toHaveAttribute("aria-expanded", "true");
  });
});
