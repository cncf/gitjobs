import { expect, test } from "@playwright/test";
import {
  navigateHomeWithRetries,
  navigateWithRetries,
  waitForActionResponse,
} from "../../shared/navigation.js";
import { countVisibleNoDataMessages } from "./helpers.js";

test.describe("GitJobs - Stats", () => {
  test.describe("with JavaScript disabled", () => {
    test.use({ javaScriptEnabled: false });

    test("exposes the server-rendered chart loading state", async ({ page }) => {
      // Load stats without allowing client-side chart initialization.
      await navigateWithRetries(page, "/stats", "public stats page");
      await expect(page).toHaveURL(/\/stats/);

      // Find every chart container and loading indicator.
      const chartContainers = page.locator("#line-chart, #bar-daily, #bar-monthly");
      const chartLoadingIndicators = page.locator("[data-chart-loading]");

      // Verify each chart exposes its server-rendered loading state.
      await expect(chartContainers).toHaveCount(3);
      await expect(chartLoadingIndicators).toHaveCount(3);
      for (const chartContainer of await chartContainers.all()) {
        await expect(chartContainer).toHaveAttribute("aria-busy", "true");
      }
      for (const loadingIndicator of await chartLoadingIndicators.all()) {
        await expect(loadingIndicator).toBeVisible();
      }
    });
  });

  test("navigates to the stats page and interacts with charts", async ({ page, browserName }) => {
    // Load the public home page before using its primary navigation.
    await navigateHomeWithRetries(page);

    // Navigate through the primary Stats link and wait for its cold response.
    const primaryNavigation = page.getByRole("navigation", { name: "Main navigation" });
    const statsLink = primaryNavigation.getByRole("link", { name: "Stats", exact: true });
    await waitForActionResponse(page, () => statsLink.click(), {
      method: "GET",
      urlEndsWith: "/stats",
    });
    await expect(page).toHaveURL(/\/stats/);

    // Wait for every chart to clear its loading state.
    const chartContainers = page.locator("#line-chart, #bar-daily, #bar-monthly");
    const chartLoadingIndicators = page.locator("[data-chart-loading]");
    await expect(chartContainers).toHaveCount(3);
    await expect(chartLoadingIndicators).toHaveCount(3);
    for (const loadingIndicator of await chartLoadingIndicators.all()) {
      await expect(loadingIndicator).toBeHidden({ timeout: 15000 });
    }
    for (const chartContainer of await chartContainers.all()) {
      await expect(chartContainer).toHaveAttribute("aria-busy", "false");
    }

    // Resolve whether the seeded database contains chart data.
    const noDataVisibleCount = await countVisibleNoDataMessages(page);

    // Verify and stop when the page exposes its valid empty state.
    if (noDataVisibleCount > 0) {
      await expect(page.getByText("No data available yet").first()).toBeVisible();
      return;
    }

    // Verify representative charts finish rendering.
    await expect(page.locator("#line-chart")).toBeVisible({ timeout: 15000 });
    await expect(page.locator("#bar-daily")).toBeVisible({ timeout: 15000 });

    // Exercise chart hit areas where the browser renderer supports it.
    if (browserName !== "firefox") {
      const lineChartTarget = page.locator("#line-chart canvas, #line-chart rect, #line-chart path").first();
      if ((await lineChartTarget.count()) > 0) {
        await lineChartTarget.click({ force: true });
      } else {
        await page.locator("#line-chart").click({ force: true });
      }

      // Exercise the daily bar chart hit area.
      const barDailyTarget = page.locator("#bar-daily canvas, #bar-daily rect, #bar-daily path").first();
      if ((await barDailyTarget.count()) > 0) {
        await barDailyTarget.click({ force: true });
      } else {
        await page.locator("#bar-daily").click({ force: true });
      }
    }
  });
});
