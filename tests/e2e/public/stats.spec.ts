import { test, expect } from "@playwright/test";
import {
  navigateHomeWithRetries,
  countVisibleNoDataMessages,
} from "../shared/helpers";

test.describe("GitJobs - Stats", () => {
  test.beforeEach(async ({ page }) => {
    await navigateHomeWithRetries(page);
  });

  test("should show loading indicators until charts render", async ({ page }) => {
    // Pause chart initialization to verify the server-rendered loading state
    let releaseStatsScript = () => {};
    const statsScriptBlocked = new Promise<void>((resolve) => {
      releaseStatsScript = () => resolve();
    });

    await page.route("**/static/js/jobboard/stats.js", async (route) => {
      await statsScriptBlocked;
      await route.continue();
    });

    const navigationPromise = page.getByRole("link", { name: "Stats" }).click();
    await page.waitForURL(/\/stats/);

    const chartContainers = page.locator("#line-chart, #bar-daily, #bar-monthly");
    const chartLoadingIndicators = page.locator("[data-chart-loading]");
    await expect(chartContainers).toHaveCount(3);
    await expect(chartLoadingIndicators).toHaveCount(3);
    for (const chartContainer of await chartContainers.all()) {
      await expect(chartContainer).toHaveAttribute("aria-busy", "true");
    }
    for (const loadingIndicator of await chartLoadingIndicators.all()) {
      await expect(loadingIndicator).toBeVisible();
    }

    // Resume chart initialization and verify every chart reaches its ready state
    releaseStatsScript();
    await navigationPromise;

    for (const loadingIndicator of await chartLoadingIndicators.all()) {
      await expect(loadingIndicator).toBeHidden({ timeout: 15000 });
    }
    for (const chartContainer of await chartContainers.all()) {
      await expect(chartContainer).toHaveAttribute("aria-busy", "false");
    }
  });

  test("should navigate to the stats page and interact with charts", async ({
    page,
    browserName,
  }) => {
    await page.getByRole("link", { name: "Stats" }).click();
    await expect(page).toHaveURL(/\/stats/);

    await expect
      .poll(
        async () => {
          const noDataVisibleCount = await countVisibleNoDataMessages(page);
          return (
            noDataVisibleCount > 0 ||
            (await page.locator("#line-chart").isVisible())
          );
        },
        { timeout: 15000 },
      )
      .toBe(true);

    const noDataVisibleCount = await countVisibleNoDataMessages(page);

    if (noDataVisibleCount > 0) {
      await expect(
        page.getByText("No data available yet").first(),
      ).toBeVisible();
      return;
    }

    await expect(page.locator("#line-chart")).toBeVisible({ timeout: 15000 });
    await expect(page.locator("#bar-daily")).toBeVisible({ timeout: 15000 });

    if (browserName === "firefox") {
      await expect(page.locator("#line-chart")).toBeVisible({ timeout: 15000 });
      await expect(page.locator("#bar-daily")).toBeVisible({ timeout: 15000 });
    } else {
      const lineChartTarget = page
        .locator("#line-chart canvas, #line-chart rect, #line-chart path")
        .first();
      if ((await lineChartTarget.count()) > 0) {
        await lineChartTarget.click({ force: true });
      } else {
        await page.locator("#line-chart").click({ force: true });
      }

      const barDailyTarget = page
        .locator("#bar-daily canvas, #bar-daily rect, #bar-daily path")
        .first();
      if ((await barDailyTarget.count()) > 0) {
        await barDailyTarget.click({ force: true });
      } else {
        await page.locator("#bar-daily").click({ force: true });
      }
    }
  });
});
