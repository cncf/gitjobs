import { test, expect } from "@playwright/test";
import {
  navigateHomeWithRetries,
  countVisibleNoDataMessages,
} from "../shared/helpers";

test.describe("GitJobs - Stats", () => {
  test.beforeEach(async ({ page }) => {
    await navigateHomeWithRetries(page);
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
