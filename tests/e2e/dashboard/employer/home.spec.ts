import { test, expect } from "@playwright/test";
import { loginWithCredentials } from "../../shared/utils";
import { navigateHomeWithRetries } from "../../shared/helpers";

test.describe("GitJobs - Employer Home", () => {
  test.beforeEach(async ({ page }) => {
    await navigateHomeWithRetries(page);
  });

  test("should close employer selector dropdown after choosing an employer", async ({
    page,
  }) => {
    await loginWithCredentials(page, "test", "test1234");
    await page.goto("/dashboard/employer");

    const employerButton = page.locator("#employer-btn");
    await expect(employerButton).toHaveCount(1);

    const dropdown = page.locator("#dropdown-employers");
    await employerButton.click();
    await expect(dropdown).toBeVisible();
    await expect(employerButton).toHaveAttribute("aria-expanded", "true");
    await expect(dropdown).toHaveAttribute("aria-hidden", "false");

    const selectableEmployers = dropdown.locator(
      "button.employer-button:not([disabled])",
    );
    expect(await selectableEmployers.count()).toBeGreaterThan(0);

    await selectableEmployers.first().click();

    await expect(dropdown).toBeHidden();
    await expect(employerButton).toHaveAttribute("aria-expanded", "false");
    await expect(dropdown).toHaveAttribute("aria-hidden", "true");
  });

  test("should close employer selector dropdown on Escape", async ({
    page,
  }) => {
    await loginWithCredentials(page, "test", "test1234");
    await page.goto("/dashboard/employer");

    const employerButton = page.locator("#employer-btn");
    await expect(employerButton).toHaveCount(1);

    const dropdown = page.locator("#dropdown-employers");
    await employerButton.click();
    await expect(dropdown).toBeVisible();
    await expect(employerButton).toHaveAttribute("aria-expanded", "true");
    await expect(dropdown).toHaveAttribute("aria-hidden", "false");

    await page.keyboard.press("Escape");

    await expect(dropdown).toBeHidden();
    await expect(employerButton).toBeFocused();
    await expect(employerButton).toHaveAttribute("aria-expanded", "false");
    await expect(dropdown).toHaveAttribute("aria-hidden", "true");
  });

  test("should close employer selector dropdown on outside click", async ({
    page,
  }) => {
    await loginWithCredentials(page, "test", "test1234");
    await page.goto("/dashboard/employer");

    const employerButton = page.locator("#employer-btn");
    await expect(employerButton).toHaveCount(1);

    const dropdown = page.locator("#dropdown-employers");
    await employerButton.click();
    await expect(dropdown).toBeVisible();
    await expect(employerButton).toHaveAttribute("aria-expanded", "true");
    await expect(dropdown).toHaveAttribute("aria-hidden", "false");

    await page
      .locator("#dashboard-content")
      .first()
      .click({
        position: { x: 8, y: 8 },
      });

    await expect(dropdown).toBeHidden();
    await expect(employerButton).toHaveAttribute("aria-expanded", "false");
    await expect(dropdown).toHaveAttribute("aria-hidden", "true");
  });
});
