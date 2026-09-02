import { expect, test } from "../../../fixtures.js";
import { navigateWithRetries } from "../../../shared/navigation.js";

test.describe("GitJobs - Employer Applications", () => {
  test("closes applications jobs dropdown after selecting a filter", async ({ employerPage: page }) => {
    // Load the employer applications dashboard.
    await navigateWithRetries(
      page,
      "/dashboard/employer?tab=applications",
      "employer applications dashboard",
    );

    // Find the jobs filter trigger and verify it is ready.
    const jobsButton = page.locator("#jobs-btn");
    await expect(jobsButton).toHaveCount(1);
    await expect(jobsButton).toBeEnabled();

    // Open the jobs dropdown and verify its accessible state.
    const dropdown = page.locator("#dropdown-jobs");
    await jobsButton.click();
    await expect(dropdown).toBeVisible();
    await expect(jobsButton).toHaveAttribute("aria-expanded", "true");
    await expect(dropdown).toHaveAttribute("aria-hidden", "false");

    // Find an available job filter.
    const selectableFilters = dropdown.locator("button:not([disabled])");
    expect(await selectableFilters.count()).toBeGreaterThan(0);

    // Select the first available filter.
    await selectableFilters.first().click();

    // Verify selection closes the dropdown and resets its state.
    await expect(dropdown).toBeHidden();
    await expect(jobsButton).toHaveAttribute("aria-expanded", "false");
    await expect(dropdown).toHaveAttribute("aria-hidden", "true");
  });

  test("closes applications jobs dropdown on Escape", async ({ employerPage: page }) => {
    // Load the employer applications dashboard.
    await navigateWithRetries(
      page,
      "/dashboard/employer?tab=applications",
      "employer applications dashboard",
    );

    // Find the jobs filter trigger and verify it is ready.
    const jobsButton = page.locator("#jobs-btn");
    await expect(jobsButton).toHaveCount(1);
    await expect(jobsButton).toBeEnabled();

    // Open the jobs dropdown and verify its accessible state.
    const dropdown = page.locator("#dropdown-jobs");
    await jobsButton.click();
    await expect(dropdown).toBeVisible();
    await expect(jobsButton).toHaveAttribute("aria-expanded", "true");
    await expect(dropdown).toHaveAttribute("aria-hidden", "false");

    // Close the dropdown with Escape.
    await page.keyboard.press("Escape");

    // Verify the trigger regains focus and exposes the closed state.
    await expect(dropdown).toBeHidden();
    await expect(jobsButton).toBeFocused();
    await expect(jobsButton).toHaveAttribute("aria-expanded", "false");
    await expect(dropdown).toHaveAttribute("aria-hidden", "true");
  });

  test("closes applications jobs dropdown on outside click", async ({ employerPage: page }) => {
    // Load the employer applications dashboard.
    await navigateWithRetries(
      page,
      "/dashboard/employer?tab=applications",
      "employer applications dashboard",
    );

    // Find the jobs filter trigger and verify it is ready.
    const jobsButton = page.locator("#jobs-btn");
    await expect(jobsButton).toHaveCount(1);
    await expect(jobsButton).toBeEnabled();

    // Open the jobs dropdown and verify its accessible state.
    const dropdown = page.locator("#dropdown-jobs");
    await jobsButton.click();
    await expect(dropdown).toBeVisible();
    await expect(jobsButton).toHaveAttribute("aria-expanded", "true");
    await expect(dropdown).toHaveAttribute("aria-hidden", "false");

    // Dismiss the dropdown from outside its interactive region.
    await page
      .locator("#dashboard-content")
      .first()
      .click({
        position: { x: 8, y: 8 },
      });

    // Verify outside dismissal resets the dropdown state.
    await expect(dropdown).toBeHidden();
    await expect(jobsButton).toHaveAttribute("aria-expanded", "false");
    await expect(dropdown).toHaveAttribute("aria-hidden", "true");
  });
});
