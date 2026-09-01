import { test, expect } from "@playwright/test";
import {
  countVisibleNoDataMessages,
  navigateHomeWithRetries,
  navigateWithRetries,
} from "../shared/helpers";

test.describe("GitJobs - Stats", () => {
  test("should show loading indicators until charts render", async ({
    baseURL,
    browser,
    page,
  }) => {
    if (!baseURL) {
      throw new Error("Playwright baseURL is required for the stats test");
    }

    const loadingContext = await browser.newContext({
      baseURL,
      javaScriptEnabled: false,
    });
    try {
      const loadingPage = await loadingContext.newPage();
      await navigateWithRetries(loadingPage, "/stats", "stats page");

      const loadingChartContainers = loadingPage.locator(
        "#line-chart, #bar-daily, #bar-monthly",
      );
      const loadingIndicators = loadingPage.locator("[data-chart-loading]");
      await expect(loadingChartContainers).toHaveCount(3);
      await expect(loadingIndicators).toHaveCount(3);
      for (const chartContainer of await loadingChartContainers.all()) {
        await expect(chartContainer).toHaveAttribute("aria-busy", "true");
      }
      for (const loadingIndicator of await loadingIndicators.all()) {
        await expect(loadingIndicator).toBeVisible();
      }
    } finally {
      await loadingContext.close();
    }

    await navigateWithRetries(page, "/stats", "stats page");
    const chartContainers = page.locator(
      "#line-chart, #bar-daily, #bar-monthly",
    );
    const chartLoadingIndicators = page.locator("[data-chart-loading]");
    await expect(chartContainers).toHaveCount(3);
    await expect(chartLoadingIndicators).toHaveCount(3);
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
    await navigateHomeWithRetries(page);

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
