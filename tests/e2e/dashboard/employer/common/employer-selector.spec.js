import { expect, test } from "../../../fixtures.js";
import { navigateWithRetries } from "../../../shared/navigation.js";

test.describe("GitJobs - Employer selector", () => {
  test("closes employer selector dropdown after choosing an employer", async ({ employerPage: page }) => {
    // Load the employer dashboard.
    await navigateWithRetries(page, "/dashboard/employer", "employer dashboard");

    // Find the employer selector trigger.
    const employerButton = page.locator("#employer-btn");
    await expect(employerButton).toHaveCount(1);

    // Open the employer selector and verify its accessible state.
    const dropdown = page.locator("#dropdown-employers");
    await employerButton.click();
    await expect(dropdown).toBeVisible();
    await expect(employerButton).toHaveAttribute("aria-expanded", "true");
    await expect(dropdown).toHaveAttribute("aria-hidden", "false");

    // Find an available employer option.
    const selectableEmployers = dropdown.locator("button.employer-button:not([disabled])");
    expect(await selectableEmployers.count()).toBeGreaterThan(0);

    // Select the first available employer.
    await selectableEmployers.first().click();

    // Verify selection closes the employer selector.
    await expect(dropdown).toBeHidden();
    await expect(employerButton).toHaveAttribute("aria-expanded", "false");
    await expect(dropdown).toHaveAttribute("aria-hidden", "true");
  });

  test("closes employer selector dropdown on Escape", async ({ employerPage: page }) => {
    // Load the employer dashboard.
    await navigateWithRetries(page, "/dashboard/employer", "employer dashboard");

    // Find the employer selector trigger.
    const employerButton = page.locator("#employer-btn");
    await expect(employerButton).toHaveCount(1);

    // Open the employer selector and verify its accessible state.
    const dropdown = page.locator("#dropdown-employers");
    await employerButton.click();
    await expect(dropdown).toBeVisible();
    await expect(employerButton).toHaveAttribute("aria-expanded", "true");
    await expect(dropdown).toHaveAttribute("aria-hidden", "false");

    // Close the employer selector with Escape.
    await page.keyboard.press("Escape");

    // Verify focus returns to the trigger and the selector is closed.
    await expect(dropdown).toBeHidden();
    await expect(employerButton).toBeFocused();
    await expect(employerButton).toHaveAttribute("aria-expanded", "false");
    await expect(dropdown).toHaveAttribute("aria-hidden", "true");
  });

  test("closes employer selector dropdown on outside click", async ({ employerPage: page }) => {
    // Load the employer dashboard.
    await navigateWithRetries(page, "/dashboard/employer", "employer dashboard");

    // Find the employer selector trigger.
    const employerButton = page.locator("#employer-btn");
    await expect(employerButton).toHaveCount(1);

    // Open the employer selector and verify its accessible state.
    const dropdown = page.locator("#dropdown-employers");
    await employerButton.click();
    await expect(dropdown).toBeVisible();
    await expect(employerButton).toHaveAttribute("aria-expanded", "true");
    await expect(dropdown).toHaveAttribute("aria-hidden", "false");

    // Dismiss the selector from outside its interactive region.
    await page
      .locator("#dashboard-content")
      .first()
      .click({
        position: { x: 8, y: 8 },
      });

    // Verify outside dismissal resets the selector state.
    await expect(dropdown).toBeHidden();
    await expect(employerButton).toHaveAttribute("aria-expanded", "false");
    await expect(dropdown).toHaveAttribute("aria-hidden", "true");
  });
});
