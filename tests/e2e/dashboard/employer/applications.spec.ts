import { test, expect } from "@playwright/test";
import { loginWithCredentials } from "../../shared/utils";
import { navigateHomeWithRetries } from "../../shared/helpers";

test.describe("GitJobs - Employer Applications", () => {
  test.beforeEach(async ({ page }) => {
    await navigateHomeWithRetries(page);
  });

  test("should close applications jobs dropdown after selecting a filter", async ({
    page,
  }) => {
    await loginWithCredentials(page, "test", "test1234");
    await page.goto("/dashboard/employer?tab=applications");

    const jobsButton = page.locator("#jobs-btn");
    await expect(jobsButton).toHaveCount(1);
    await expect(jobsButton).toBeEnabled();

    const dropdown = page.locator("#dropdown-jobs");
    await jobsButton.click();
    await expect(dropdown).toBeVisible();
    await expect(jobsButton).toHaveAttribute("aria-expanded", "true");
    await expect(dropdown).toHaveAttribute("aria-hidden", "false");

    const selectableFilters = dropdown.locator("button:not([disabled])");
    expect(await selectableFilters.count()).toBeGreaterThan(0);

    await selectableFilters.first().click();

    await expect(dropdown).toBeHidden();
    await expect(jobsButton).toHaveAttribute("aria-expanded", "false");
    await expect(dropdown).toHaveAttribute("aria-hidden", "true");
  });

  test("should close applications jobs dropdown on Escape", async ({
    page,
  }) => {
    await loginWithCredentials(page, "test", "test1234");
    await page.goto("/dashboard/employer?tab=applications");

    const jobsButton = page.locator("#jobs-btn");
    await expect(jobsButton).toHaveCount(1);
    await expect(jobsButton).toBeEnabled();

    const dropdown = page.locator("#dropdown-jobs");
    await jobsButton.click();
    await expect(dropdown).toBeVisible();
    await expect(jobsButton).toHaveAttribute("aria-expanded", "true");
    await expect(dropdown).toHaveAttribute("aria-hidden", "false");

    await page.keyboard.press("Escape");

    await expect(dropdown).toBeHidden();
    await expect(jobsButton).toBeFocused();
    await expect(jobsButton).toHaveAttribute("aria-expanded", "false");
    await expect(dropdown).toHaveAttribute("aria-hidden", "true");
  });

  test("should close applications jobs dropdown on outside click", async ({
    page,
  }) => {
    await loginWithCredentials(page, "test", "test1234");
    await page.goto("/dashboard/employer?tab=applications");

    const jobsButton = page.locator("#jobs-btn");
    await expect(jobsButton).toHaveCount(1);
    await expect(jobsButton).toBeEnabled();

    const dropdown = page.locator("#dropdown-jobs");
    await jobsButton.click();
    await expect(dropdown).toBeVisible();
    await expect(jobsButton).toHaveAttribute("aria-expanded", "true");
    await expect(dropdown).toHaveAttribute("aria-hidden", "false");

    await page
      .locator("#dashboard-content")
      .first()
      .click({
        position: { x: 8, y: 8 },
      });

    await expect(dropdown).toBeHidden();
    await expect(jobsButton).toHaveAttribute("aria-expanded", "false");
    await expect(dropdown).toHaveAttribute("aria-hidden", "true");
  });
});
