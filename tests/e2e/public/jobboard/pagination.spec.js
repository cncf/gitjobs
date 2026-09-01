import { expect, test } from "@playwright/test";

import { navigateHomeWithRetries, navigateWithRetries } from "../../shared/navigation.js";

/**
 * Opens a paginated result set with one job per page.
 * @param {import("@playwright/test").Page} page - Job board page.
 * @returns {Promise<void>}
 */
const openFirstPageWithPagination = async (page) => {
  // Load the job board with one result per page.
  await navigateWithRetries(page, "/?limit=1", "job board first page");

  // Verify the first result range is rendered.
  await expect(page.locator("#results")).toHaveText(/^1 - 1 of \d+ results$/, {
    timeout: 10000,
  });

  // Read and verify the seeded total supports pagination.
  const resultsText = (await page.locator("#results").textContent()) || "";
  const totalResults = Number(resultsText.match(/of\s+(\d+)\s+results/i)?.[1] || "0");
  expect(totalResults).toBeGreaterThan(1);
  // Verify the forward pagination control is available.
  await expect(page.getByRole("link", { name: "Next" })).toBeVisible();
};

test.describe("GitJobs - Jobboard pagination", () => {
  test.beforeEach(async ({ page }) => {
    // Load the public job board before each pagination assertion.
    await navigateHomeWithRetries(page);
  });

  test("paginates through jobs", async ({ page }) => {
    // Prepare a result set with one job per page.
    await openFirstPageWithPagination(page);

    // Follow the next-page control.
    await page.getByRole("link", { name: "Next" }).click();

    // Verify the URL and visible result range advance together.
    await expect(page).toHaveURL(/limit=1/);
    await expect(page).toHaveURL(/offset=1/);
    await expect(page.locator("#results")).toHaveText(/^2 - 2 of \d+ results$/);
  });

  test("shows a spinner while loading the next page", async ({ page }) => {
    // Prepare a result set with one job per page.
    await openFirstPageWithPagination(page);

    // Delay the next result request so its loading state is observable.
    let delayed = false;
    await page.route("**/section/jobs/results*", async (route) => {
      if (!delayed) {
        delayed = true;
        await new Promise((resolve) => setTimeout(resolve, 500));
      }
      await route.continue();
    });

    // Follow the next-page control and verify its spinner appears.
    await page.getByRole("link", { name: "Next" }).click();

    // Verify navigation completes with the next visible result range.
    await expect(page.locator("#pagination-next-spinner")).toBeVisible();
    await expect(page).toHaveURL(/offset=1/);
    await expect(page.locator("#results")).toHaveText(/^2 - 2 of \d+ results$/);
  });
});
