import { expect, test } from "@playwright/test";
import { navigateHomeWithRetries } from "../../shared/navigation.js";

test.describe("GitJobs - About", () => {
  test.beforeEach(async ({ page }) => {
    // Load the public home page before each about-page assertion.
    await navigateHomeWithRetries(page);
  });

  test("navigates to the About page and renders its content", async ({ page }) => {
    // Follow the public About navigation link.
    await page.getByRole("link", { name: "About" }).click();
    await expect(page).toHaveURL(/\/about/);

    // Find the main About content.
    const aboutContent = page.locator(".about");

    // Verify the page exposes its overview and FAQ sections.
    await expect(aboutContent).toBeVisible();
    await expect(aboutContent.getByRole("heading", { name: "ABOUT" })).toBeVisible();
    await expect(aboutContent.getByRole("heading", { name: "FAQs" })).toBeVisible();
  });
});
