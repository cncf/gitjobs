import { expect, test } from "@playwright/test";

import { navigateHomeWithRetries } from "../../shared/navigation.js";
import {
  JOB_TITLE_SELECTOR,
  jobCards,
  jobTitles,
  jobTypeButtons,
  searchInput,
  waitForOnlyJobTypeResults,
  waitForOnlyJobTypeSetResults,
} from "./helpers.js";

/**
 * Returns a desktop job-kind filter.
 * @param {import("@playwright/test").Page} page - Job board page.
 * @param {string} kind - Job kind value.
 * @returns {import("@playwright/test").Locator}
 */
const desktopKindFilter = (page, kind) => {
  return page.locator(`label[for="desktop-kind[]-${kind}"]`);
};

/**
 * Returns a mobile job-kind filter.
 * @param {import("@playwright/test").Page} page - Job board page.
 * @param {string} kind - Job kind value.
 * @returns {import("@playwright/test").Locator}
 */
const mobileKindFilter = (page, kind) => {
  return page.locator(`label[for="mobile-kind[]-${kind}"]`);
};

test.describe("GitJobs - Jobboard filters", () => {
  test.beforeEach(async ({ page }) => {
    // Load the public job board before each filter assertion.
    await navigateHomeWithRetries(page);
  });

  test("applies a filter and updates results", async ({ page }) => {
    // Apply the desktop full-time filter.
    await desktopKindFilter(page, "full-time").click();

    // Verify the URL and visible results reflect the selected job type.
    await expect(page).toHaveURL(/full-time/);
    await waitForOnlyJobTypeResults(page, "full time");

    // Verify every visible job card shows the selected type.
    for (const jobCard of await jobTypeButtons(page).all()) {
      const jobType = jobCard.locator(".capitalize").first();
      if (await jobType.isVisible()) {
        await expect(jobType).toHaveText("full time");
      }
    }
  });

  test("applies multiple filters and updates results", async ({ page }) => {
    // Apply two desktop job-type filters.
    await desktopKindFilter(page, "part-time").click();
    await desktopKindFilter(page, "internship").click();
    // Verify the URL and visible results preserve both selections.
    await expect(page).toHaveURL(/part-time/);
    await expect(page).toHaveURL(/internship/);
    await waitForOnlyJobTypeSetResults(page, ["part time", "internship"]);

    // Verify every visible job card matches one selected type.
    for (const jobCard of await jobTypeButtons(page).all()) {
      const jobType = jobCard.locator(".capitalize").first();
      if (await jobType.isVisible()) {
        expect(["part time", "internship"]).toContain((await jobType.textContent())?.trim());
      }
    }
  });

  test("omits empty and zero default filter values", async ({ page }) => {
    // Watch the result request produced by a filter change.
    const requestPromise = page.waitForRequest((request) => {
      return request.method() === "GET" && request.url().includes("/section/jobs/results");
    });

    // Apply one non-default filter.
    await desktopKindFilter(page, "full-time").click();

    // Verify the request omits empty and zero-value defaults.
    const query = new URL((await requestPromise).url()).search;
    expect(query).not.toContain("seniority=");
    expect(query).not.toContain("open_source=0");
    expect(query).not.toContain("upstream_commitment=0");
    expect(query).not.toContain("salary_min=0");
    expect(query).not.toContain("ts_query=");
  });

  test("searches jobs and updates results", async ({ page }) => {
    // Submit a public job-title search.
    await searchInput(page).fill("Engineer");
    await page.locator("#search-jobs-btn").click();

    // Wait until every rendered title reflects the search term.
    await page.waitForFunction(
      ({ selector, term }) => {
        const titles = Array.from(document.querySelectorAll(selector));
        return titles.length > 0 && titles.every((title) => title.textContent?.toLowerCase().includes(term));
      },
      { selector: JOB_TITLE_SELECTOR, term: "engineer" },
    );

    // Verify each visible title contains the normalized search term.
    for (const title of await jobTitles(page).allTextContents()) {
      expect(title.trim().toLowerCase()).toContain("engineer");
    }
  });

  test("applies and resets a mobile filter @mobile", async ({ page }) => {
    // Find the mobile filter trigger and active-filter indicator.
    const filtersIndicator = page.locator("#mobile-filters-indicator");
    const openFiltersButton = page.locator("#open-filters");
    // Verify the initial mobile filter state is empty.
    await expect(filtersIndicator).toBeHidden();
    await expect(openFiltersButton).toHaveAttribute("aria-label", "Open filters");

    // Apply a full-time filter through the mobile drawer.
    await openFiltersButton.click();
    await expect(page.locator("#drawer-filters")).toBeVisible();
    await mobileKindFilter(page, "full-time").click();
    await page.locator("#close-filters").click();
    await expect(page).toHaveURL(/full-time/);
    await expect(filtersIndicator).toBeVisible();
    await expect(openFiltersButton).toHaveAttribute("aria-label", "Open filters, filters selected");
    await waitForOnlyJobTypeResults(page, "full time");

    // Reopen the drawer and reset every mobile filter.
    await openFiltersButton.click();
    const filtersDrawer = page.locator("#drawer-filters");
    await expect(filtersDrawer).toHaveAttribute("data-open", "true");
    await page.locator("#reset-mobile-filters").click();
    await expect(filtersDrawer).toHaveAttribute("aria-hidden", "true");
    await expect(filtersDrawer).toHaveAttribute("data-open", "false");
    await expect(openFiltersButton).toBeFocused();
    await expect(page).not.toHaveURL(/full-time/);
    await expect(filtersIndicator).toBeHidden();
  });

  test("closes the mobile filters drawer on Escape @mobile", async ({ page }) => {
    // Open the mobile filters drawer.
    const openFiltersButton = page.locator("#open-filters");
    await openFiltersButton.click();

    // Find the drawer and verify its accessible open state.
    const drawer = page.locator("#drawer-filters");
    await expect(drawer).toHaveAttribute("data-open", "true");
    await expect(drawer).toHaveAttribute("aria-hidden", "false");

    // Close the drawer with Escape and verify its accessible state.
    await page.keyboard.press("Escape");
    await expect(drawer).toHaveAttribute("data-open", "false");
    await expect(drawer).toHaveAttribute("aria-hidden", "true");
  });

  test("resets desktop filters", async ({ page }) => {
    // Apply and verify a desktop part-time filter.
    await desktopKindFilter(page, "part-time").click();
    await waitForOnlyJobTypeResults(page, "part time");
    // Reset every desktop filter.
    await page.locator("#reset-desktop-filters").click();

    // Verify the URL, result set, and result count return to defaults.
    await expect(page).not.toHaveURL(/part-time/);
    await expect(jobCards(page).first()).toBeVisible();
    await expect
      .poll(async () => {
        const totalCards = await jobCards(page).count();
        const partTimeCards = await page
          .locator('[data-preview-job="true"]', {
            hasText: /Job\s*type\s*part\s*time/i,
          })
          .count();
        return totalCards > 0 && partTimeCards < totalCards;
      })
      .toBe(true);
    await expect(page.locator("#results")).toHaveText(/^1 - \d+ of \d+ results$/);
  });

  test("sorts jobs by salary", async ({ page }) => {
    // Capture the initial visible job order.
    const initialJobIds = await jobCards(page).evaluateAll((cards) => {
      return cards.map((card) => card.getAttribute("data-job-id") || "");
    });

    // Apply salary sorting and verify its URL contract.
    await page.locator("#sort-desktop").selectOption("salary");
    await expect(page).toHaveURL(/\?sort=salary/);

    // Capture and verify the updated visible job order.
    const sortedJobIds = await jobCards(page).evaluateAll((cards) => {
      return cards.map((card) => card.getAttribute("data-job-id") || "");
    });
    expect(sortedJobIds).toHaveLength(initialJobIds.length);
    expect(sortedJobIds.some((jobId, index) => jobId !== initialJobIds[index])).toBe(true);

    // Verify missing salaries remain grouped after provided salaries.
    const salaryValues = await page
      .locator('[data-preview-job="true"] [data-testid="job-card-salary"]')
      .allTextContents();
    expect(salaryValues).toHaveLength(sortedJobIds.length);
    let foundMissingSalary = false;
    for (const salaryText of salaryValues) {
      if (/not provided/i.test(salaryText)) {
        foundMissingSalary = true;
        continue;
      }
      expect(foundMissingSalary).toBe(false);
    }
  });

  test("persists filters and search after refresh", async ({ page }) => {
    // Apply a search term and desktop job-type filter.
    await searchInput(page).fill("Engineer");
    await desktopKindFilter(page, "full-time").click();
    await expect(page).toHaveURL(/Engineer/);
    await expect(page).toHaveURL(/full-time/);

    // Refresh the current result URL.
    const urlBeforeRefresh = page.url();
    await page.reload();
    // Verify the URL and filter controls preserve their state.
    await expect(page).toHaveURL(urlBeforeRefresh);
    await expect(searchInput(page)).toHaveValue("Engineer");
    await expect(page.locator('input[id="desktop-kind[]-full-time"]')).toBeChecked();
  });
});
