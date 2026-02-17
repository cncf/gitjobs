import { test, expect } from "@playwright/test";
import { navigateHomeWithRetries } from "../shared/helpers";

test.describe("GitJobs - About", () => {
  test.beforeEach(async ({ page }) => {
    await navigateHomeWithRetries(page);
  });

  test("should navigate to the about page and check for a body", async ({
    page,
  }) => {
    await page.getByRole("link", { name: "About" }).click();
    await expect(page).toHaveURL(/\/about/);
    await expect(page.locator("body")).toBeVisible();
  });
});
