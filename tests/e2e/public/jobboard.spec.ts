import { test, expect, Page } from "@playwright/test";
import {
  jobCards,
  jobTitles,
  jobTypeButtons,
  searchInput,
  JOB_TITLE_SELECTOR,
} from "../shared/utils";
import {
  navigateHomeWithRetries,
  waitForOnlyJobTypeResults,
  waitForOnlyJobTypeSetResults,
} from "../shared/helpers";

test.describe("GitJobs - Jobboard", () => {
  const openFirstPageWithPagination = async (page: Page): Promise<void> => {
    await page.goto("/?limit=1");
    await expect(page.locator("#results")).toHaveText(
      /^1 - 1 of \d+ results$/,
      {
        timeout: 10000,
      },
    );

    const resultsText = (await page.locator("#results").textContent()) || "";
    const totalResultsMatch = resultsText.match(/of\s+(\d+)\s+results/i);
    const totalResults = Number(totalResultsMatch?.[1] || "0");
    expect(totalResults).toBeGreaterThan(1);

    await expect(page.getByRole("link", { name: "Next" })).toBeVisible({
      timeout: 10000,
    });
  };

  test.beforeEach(async ({ page }) => {
    await navigateHomeWithRetries(page);
  });

  test("should have the correct title and heading", async ({ page }) => {
    await expect(page).toHaveTitle(/GitJobs/);
    await expect(page.getByRole("heading", { level: 1 })).toBeVisible();
  });

  test("should apply a filter and verify that the results are updated", async ({
    page,
  }) => {
    await page
      .locator("div:nth-child(4) > div > .font-semibold")
      .first()
      .click();
    await page.locator("label").filter({ hasText: "Full Time" }).nth(1).click();
    await expect(page).toHaveURL(/full-time/);
    await waitForOnlyJobTypeResults(page, "full time");

    const jobTypeButtonsList = await jobTypeButtons(page).all();
    for (const jobCard of jobTypeButtonsList) {
      const jobTypeElement = jobCard.locator(".capitalize").first();
      if (await jobTypeElement.isVisible()) {
        await expect(jobTypeElement).toHaveText("full time");
      }
    }
  });

  test("should apply multiple filters and verify that the results are updated", async ({
    page,
  }) => {
    await page
      .locator("div:nth-child(4) > div > .font-semibold")
      .first()
      .click();
    await page.locator("label").filter({ hasText: "Part Time" }).nth(1).click();
    await page
      .locator("label")
      .filter({ hasText: "Internship" })
      .nth(1)
      .click();
    await expect(page).toHaveURL(/part-time/);
    await expect(page).toHaveURL(/internship/);
    await waitForOnlyJobTypeSetResults(page, ["part time", "internship"]);

    const jobTypeButtonsList = await jobTypeButtons(page).all();
    for (const jobCard of jobTypeButtonsList) {
      const jobTypeElement = jobCard.locator(".capitalize").first();
      if (await jobTypeElement.isVisible()) {
        const jobTypeText = await jobTypeElement.textContent();
        expect(["part time", "internship"]).toContain(jobTypeText?.trim());
      }
    }
  });

  test("should not send empty or zero default filter values", async ({
    page,
  }) => {
    const requestPromise = page.waitForRequest((request) => {
      return (
        request.method() === "GET" &&
        request.url().includes("/section/jobs/results")
      );
    });

    await page.locator("label").filter({ hasText: "Full Time" }).nth(1).click();

    const requestUrl = (await requestPromise).url();
    const query = new URL(requestUrl).search;

    expect(query).not.toContain("seniority=");
    expect(query).not.toContain("open_source=0");
    expect(query).not.toContain("upstream_commitment=0");
    expect(query).not.toContain("salary_min=0");
    expect(query).not.toContain("ts_query=");
  });

  test("should search for a job and verify that the results are updated and contain the search term", async ({
    page,
  }) => {
    await searchInput(page).click();
    await searchInput(page).fill("Engineer");
    await page.locator("#search-jobs-btn").click();

    await page.waitForFunction(
      ({ selector, term }) => {
        const nodes = Array.from(document.querySelectorAll(selector));
        if (nodes.length === 0) {
          return false;
        }
        return nodes.every((node) =>
          node.textContent?.toLowerCase().includes(term),
        );
      },
      { selector: JOB_TITLE_SELECTOR, term: "engineer" },
    );

    const jobTitleValues = await jobTitles(page).allTextContents();
    for (const title of jobTitleValues) {
      expect(title.trim().toLowerCase()).toContain("engineer");
    }
  });

  test("should apply a filter and verify that the results are updated on mobile", async ({
    page,
  }) => {
    await page.setViewportSize({ width: 375, height: 667 });
    await page.locator("#open-filters").click();
    await page.waitForSelector("#drawer-filters", { state: "visible" });
    await page
      .locator("#drawer-filters label")
      .filter({ hasText: "Full Time" })
      .click();
    await page.locator("#close-filters").click();
    await expect(page).toHaveURL(/full-time/);
    await waitForOnlyJobTypeResults(page, "full time");

    const jobTypeButtonsList = await jobTypeButtons(page).all();
    for (const jobCard of jobTypeButtonsList) {
      const jobTypeElement = jobCard.locator(".capitalize").first();
      if (await jobTypeElement.isVisible()) {
        await expect(jobTypeElement).toHaveText("full time");
      }
    }
  });

  test("should close mobile filters drawer on Escape", async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 667 });
    const openFiltersButton = page.locator("#open-filters");
    await expect(openFiltersButton).toBeVisible();

    await openFiltersButton.click();

    const drawer = page.locator("#drawer-filters");
    await expect(drawer).toHaveAttribute("data-open", "true");
    await expect(drawer).toHaveAttribute("aria-hidden", "false");

    await page.keyboard.press("Escape");

    await expect(drawer).toHaveAttribute("data-open", "false");
    await expect(drawer).toHaveAttribute("aria-hidden", "true");
  });

  test("should reset filters", async ({ page }) => {
    await page.locator("label").filter({ hasText: "Part Time" }).nth(1).click();
    await expect(page).toHaveURL(/part-time/);
    await waitForOnlyJobTypeResults(page, "part time");
    await page.locator("#reset-desktop-filters").click();

    await expect(page).not.toHaveURL(/part-time/);
    await expect(
      page.locator('[data-preview-job="true"]').first(),
    ).toBeVisible();
    await expect
      .poll(async () => {
        const cards = page.locator('[data-preview-job="true"]');
        const totalCards = await cards.count();
        if (totalCards === 0) {
          return false;
        }

        const partTimeCards = await page
          .locator('[data-preview-job="true"]', {
            hasText: /Job\s*type\s*part\s*time/i,
          })
          .count();

        return partTimeCards < totalCards;
      })
      .toBe(true);

    await expect(page.locator("#results")).toHaveText(
      /^1 - \d+ of \d+ results$/,
    );
  });

  test("should sort jobs", async ({ page }) => {
    const initialJobTitles = (await jobTitles(page).allTextContents()).map(
      (title) => title.trim(),
    );
    await page.locator("#sort-desktop").selectOption("salary");
    await expect(page).toHaveURL(/\?sort=salary/);
    const sortedJobTitles = (await jobTitles(page).allTextContents()).map(
      (title) => title.trim(),
    );
    expect(sortedJobTitles.length).toBe(initialJobTitles.length);
    expect(sortedJobTitles[0]).toBeTruthy();
    expect(
      sortedJobTitles.some((title, index) => title !== initialJobTitles[index]),
    ).toBe(true);
  });

  test("ensure filters and search persist on page refresh", async ({
    page,
  }) => {
    await searchInput(page).fill("Engineer");
    await page.locator("label").filter({ hasText: "Full Time" }).nth(1).click();
    await expect(page).toHaveURL(/Engineer/);
    await expect(page).toHaveURL(/full-time/);

    const urlBeforeRefresh = page.url();
    expect(urlBeforeRefresh).toContain("Engineer");
    expect(urlBeforeRefresh).toContain("full-time");

    await page.reload();
    await page.waitForLoadState("domcontentloaded");

    const urlAfterRefresh = page.url();
    expect(urlAfterRefresh).toBe(urlBeforeRefresh);

    await expect(searchInput(page)).toHaveValue("Engineer");
    await expect(
      page.locator('input[id="desktop-kind[]-full-time"]'),
    ).toBeChecked();
  });

  test("should show hover states and preview on job card interactions", async ({
    page,
  }) => {
    await jobCards(page).first().waitFor();
    const firstJobCard = jobCards(page).first();

    const jobTitle = await firstJobCard
      .locator(JOB_TITLE_SELECTOR)
      .textContent();
    expect(jobTitle?.trim()).toBeTruthy();

    await firstJobCard.hover();
    await expect(firstJobCard).toBeVisible();
    await expect(page.locator("#preview-modal")).not.toBeVisible();
  });

  test("should display job details correctly", async ({ page }) => {
    const selectedJobCard = jobCards(page).first();
    await selectedJobCard.waitFor();

    const expectedTitle = (
      await selectedJobCard.getAttribute("data-job-title")
    )?.trim();
    expect(expectedTitle).toBeTruthy();

    const expectedCompany = (
      await selectedJobCard.getAttribute("data-job-company")
    )?.trim();
    expect(expectedCompany).toBeTruthy();

    await selectedJobCard.click();
    await expect(page.locator("#preview-modal .text-xl")).toBeVisible({
      timeout: 10000,
    });

    await expect(
      page.locator(
        ".text-xl.lg\\:leading-tight.font-stretch-condensed.font-medium.text-stone-900.lg\\:truncate.my-1\\.5.md\\:my-0",
      ),
    ).toHaveText(expectedTitle as string);
    await expect(page.locator("#preview-content")).toContainText(
      expectedCompany as string,
    );

    const previewContent = page.locator("#preview-content");
    const descriptionLocator = previewContent.locator(
      'div.text-lg.font-semibold.text-stone-800:has-text("Job description") + div.text-sm\\/6.text-stone-600.markdown',
    );
    await expect(descriptionLocator).toBeVisible();
    await expect(descriptionLocator).not.toHaveText("");

    await expect(previewContent.getByText("Job type")).toBeVisible();
    await expect(previewContent.getByText("Workplace")).toBeVisible();
    await expect(
      previewContent.getByRole("button", { name: "Apply" }),
    ).toBeEnabled();
    await expect(previewContent.getByText(/Published/)).toBeVisible();
    await expect(previewContent.getByText("Share this job")).toBeVisible();
  });

  test("should lock body scroll while preview modal is open and restore on close", async ({
    page,
  }) => {
    await jobCards(page).first().waitFor();
    await jobCards(page).first().click();
    await expect(page.locator("#preview-modal .text-xl")).toBeVisible({
      timeout: 10000,
    });

    await expect
      .poll(async () =>
        page.evaluate(() => document.body.dataset.modalOpenCount || "0"),
      )
      .toBe("1");
    await expect
      .poll(async () => page.evaluate(() => document.body.style.overflow))
      .toBe("hidden");

    await page.locator("#close-preview-modal").click();
    await expect(page.locator("#preview-modal")).toBeHidden();

    await expect
      .poll(async () =>
        page.evaluate(() => document.body.dataset.modalOpenCount || "0"),
      )
      .toBe("0");
    await expect
      .poll(async () => page.evaluate(() => document.body.style.overflow))
      .toBe("");
  });

  test("should keep body scroll locked when closing embed modal over preview modal", async ({
    page,
  }) => {
    await jobCards(page).first().waitFor();
    await jobCards(page).first().click();
    await expect(page.locator("#preview-modal .text-xl")).toBeVisible({
      timeout: 10000,
    });

    const embedButton = page.locator("#embed-code-button");
    if ((await embedButton.count()) === 0) {
      console.log("Embed code button not available, skipping test.");
      return;
    }

    await embedButton.click();
    await expect(page.locator("#embed-code-modal")).toBeVisible();

    await expect
      .poll(async () =>
        page.evaluate(() => document.body.dataset.modalOpenCount || "0"),
      )
      .toBe("2");
    await expect
      .poll(async () => page.evaluate(() => document.body.style.overflow))
      .toBe("hidden");

    await page.locator("#close-embed-code-modal").click();
    await expect(page.locator("#embed-code-modal")).toBeHidden();
    await expect(page.locator("#preview-modal .text-xl")).toBeVisible();

    await expect
      .poll(async () =>
        page.evaluate(() => document.body.dataset.modalOpenCount || "0"),
      )
      .toBe("1");
    await expect
      .poll(async () => page.evaluate(() => document.body.style.overflow))
      .toBe("hidden");

    await page.locator("#close-preview-modal").click();
    await expect(page.locator("#preview-modal")).toBeHidden();
    await expect
      .poll(async () =>
        page.evaluate(() => document.body.dataset.modalOpenCount || "0"),
      )
      .toBe("0");
    await expect
      .poll(async () => page.evaluate(() => document.body.style.overflow))
      .toBe("");
  });

  test("should display share buttons properly", async ({ page }) => {
    await jobCards(page).first().waitFor();
    await jobCards(page).first().click();
    await expect(page.locator("#preview-modal .text-xl")).toBeVisible({
      timeout: 10000,
    });

    const shareButtons = [
      { title: "Twitter share link", sharer: "twitter" },
      { title: "Facebook share link", sharer: "facebook" },
      { title: "LinkedIn share link", sharer: "linkedin" },
      { title: "Email share link", sharer: "email" },
      { title: "Copy link", sharer: "" },
    ];

    for (const button of shareButtons) {
      const element = page.getByTitle(button.title);
      await expect(element).toBeVisible();
      if (button.title !== "Copy link") {
        await expect
          .poll(async () => (await element.getAttribute("data-sharer")) || "")
          .toBe(button.sharer);
        await expect
          .poll(async () => (await element.getAttribute("data-url")) || "")
          .toContain("job_id=");
        await expect
          .poll(async () => (await element.getAttribute("href")) || "")
          .not.toBe("");
        const href = await element.getAttribute("href");
        expect(href).toBeTruthy();
        if (button.title === "Email share link") {
          expect(href).toMatch(/^mailto:/);
        } else {
          expect(href).toMatch(/^https?:\/\//);
        }
      } else {
        await expect(element).toBeEnabled();
      }
    }
  });

  test("should allow paginating through jobs", async ({ page }) => {
    await openFirstPageWithPagination(page);

    const nextButton = page.getByRole("link", { name: "Next" });
    await nextButton.click();
    await expect(page).toHaveURL(/limit=1/);
    await expect(page).toHaveURL(/offset=1/);
    await expect(page.locator("#results")).toHaveText(/^2 - 2 of \d+ results$/);
  });

  test("should show pagination spinner while loading next page", async ({
    page,
  }) => {
    await openFirstPageWithPagination(page);

    let delayed = false;
    await page.route("**/section/jobs/results*", async (route) => {
      if (!delayed) {
        delayed = true;
        await new Promise((resolve) => setTimeout(resolve, 500));
      }
      await route.continue();
    });

    const nextButton = page.getByRole("link", { name: "Next" });
    await nextButton.click();

    await expect(page.locator("#pagination-next-spinner")).toBeVisible();
    await expect(page).toHaveURL(/limit=1/);
    await expect(page).toHaveURL(/offset=1/);
    await expect(page.locator("#results")).toHaveText(/^2 - 2 of \d+ results$/);
  });
});
