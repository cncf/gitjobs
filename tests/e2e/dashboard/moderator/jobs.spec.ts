import { test, expect } from "@playwright/test";
import { loginWithCredentials } from "../../shared/utils";
import { navigateHomeWithRetries } from "../../shared/helpers";

test.describe("GitJobs - Moderator Jobs", () => {
  test.beforeEach(async ({ page }) => {
    await navigateHomeWithRetries(page);
  });

  test("should close moderator mobile menu drawer on Escape", async ({
    page,
  }) => {
    await loginWithCredentials(page, "test", "test1234");
    await page.setViewportSize({ width: 375, height: 667 });
    await page.goto("/dashboard/moderator", { waitUntil: "domcontentloaded" });

    const openMenuButton = page.locator("#open-menu-button");
    await expect(openMenuButton).toBeVisible();

    await openMenuButton.click();

    const drawer = page.locator("#drawer-menu");
    await expect(drawer).toHaveAttribute("data-open", "true");
    await expect(drawer).toHaveAttribute("aria-hidden", "false");

    await page.keyboard.press("Escape");

    await expect(drawer).toHaveAttribute("data-open", "false");
    await expect(drawer).toHaveAttribute("aria-hidden", "true");
  });
});
