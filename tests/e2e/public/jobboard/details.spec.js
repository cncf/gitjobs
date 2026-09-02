import { expect, test } from "@playwright/test";

import { navigateHomeWithRetries } from "../../shared/navigation.js";
import { JOB_TITLE_SELECTOR, jobCards } from "./helpers.js";

/**
 * Returns the title rendered in the job preview.
 * @param {import("@playwright/test").Page} page - Job board page.
 * @returns {import("@playwright/test").Locator}
 */
const previewTitle = (page) => page.getByTestId("preview-job-title");

test.describe("GitJobs - Jobboard details", () => {
  test.beforeEach(async ({ page }) => {
    // Load the public job board before each detail assertion.
    await navigateHomeWithRetries(page);
  });

  test("renders the expected title and heading", async ({ page }) => {
    // Verify the job board exposes its document title and primary heading.
    await expect(page).toHaveTitle(/GitJobs/);
    await expect(page.getByRole("heading", { level: 1 })).toBeVisible();
  });

  test("shows hover states on job card interactions", async ({ page }) => {
    // Find the first seeded job card.
    await jobCards(page).first().waitFor();
    const firstJobCard = jobCards(page).first();

    // Verify the card exposes a visible job title.
    const jobTitle = await firstJobCard.locator(JOB_TITLE_SELECTOR).textContent();
    expect(jobTitle?.trim()).toBeTruthy();

    // Hover the card and verify no preview opens from hover alone.
    await firstJobCard.hover();
    await expect(firstJobCard).toBeVisible();
    await expect(page.locator("#preview-modal")).toBeHidden();
  });

  test("displays job details", async ({ page }) => {
    // Find the first seeded job card.
    const selectedJobCard = jobCards(page).first();
    await selectedJobCard.waitFor();

    // Read the card metadata used by the preview contract.
    const expectedTitle = (await selectedJobCard.getAttribute("data-job-title"))?.trim();
    expect(expectedTitle).toBeTruthy();
    const expectedCompany = (await selectedJobCard.getAttribute("data-job-company"))?.trim();
    expect(expectedCompany).toBeTruthy();

    // Open the job preview and verify its title.
    await selectedJobCard.click();
    await expect(previewTitle(page)).toBeVisible({ timeout: 10000 });
    await expect(previewTitle(page)).toHaveText(expectedTitle);

    // Find the preview content and verify the complete job summary.
    const previewContent = page.locator("#preview-content");
    await expect(previewContent).toContainText(expectedCompany);
    const description = previewContent.getByTestId("preview-job-description");
    await expect(description).toBeVisible();
    await expect(description).not.toHaveText("");
    await expect(previewContent.getByText("Job type")).toBeVisible();
    await expect(previewContent.getByText("Workplace")).toBeVisible();
    await expect(previewContent.getByRole("button", { name: "Apply" })).toBeEnabled();
    await expect(previewContent.getByText(/Published/)).toBeVisible();
    await expect(previewContent.getByText("Share this job")).toBeVisible();
  });

  test("locks body scroll while the preview modal is open", async ({ page }) => {
    // Open the first job preview.
    await jobCards(page).first().click();
    await expect(previewTitle(page)).toBeVisible({ timeout: 10000 });

    // Verify the preview owns one global body scroll lock.
    await expect.poll(async () => page.evaluate(() => document.body.dataset.modalOpenCount || "0")).toBe("1");
    await expect.poll(async () => page.evaluate(() => document.body.style.overflow)).toBe("hidden");

    // Close the preview and verify the global scroll lock is released.
    await page.locator("#close-preview-modal").click();
    await expect(page.locator("#preview-modal")).toBeHidden();
    await expect.poll(async () => page.evaluate(() => document.body.dataset.modalOpenCount || "0")).toBe("0");
    await expect.poll(async () => page.evaluate(() => document.body.style.overflow)).toBe("");
  });

  test("keeps scroll locked while closing a nested modal", async ({ page }) => {
    // Open the first job preview.
    await jobCards(page).first().click();
    await expect(previewTitle(page)).toBeVisible({ timeout: 10000 });

    // Find the optional embed-code action.
    const embedButton = page.locator("#embed-code-button");
    test.skip((await embedButton.count()) === 0, "Embed code button not available for the selected job.");

    // Open the nested embed modal and verify both scroll locks are active.
    await embedButton.click();
    await expect(page.locator("#embed-code-modal")).toBeVisible();
    await expect.poll(async () => page.evaluate(() => document.body.dataset.modalOpenCount || "0")).toBe("2");

    // Close the nested modal and verify the preview lock remains active.
    await page.locator("#close-embed-code-modal").click();
    await expect(page.locator("#embed-code-modal")).toBeHidden();
    await expect(previewTitle(page)).toBeVisible();
    await expect.poll(async () => page.evaluate(() => document.body.dataset.modalOpenCount || "0")).toBe("1");
    await expect.poll(async () => page.evaluate(() => document.body.style.overflow)).toBe("hidden");

    // Close the preview and verify every scroll lock is released.
    await page.locator("#close-preview-modal").click();
    await expect(page.locator("#preview-modal")).toBeHidden();
    await expect.poll(async () => page.evaluate(() => document.body.dataset.modalOpenCount || "0")).toBe("0");
    await expect.poll(async () => page.evaluate(() => document.body.style.overflow)).toBe("");
  });

  test("displays share buttons", async ({ page }) => {
    // Open the first job preview.
    await jobCards(page).first().click();
    await expect(previewTitle(page)).toBeVisible({ timeout: 10000 });

    // Set up every expected sharing destination.
    const shareButtons = [
      { title: "Twitter share link", sharer: "twitter" },
      { title: "Facebook share link", sharer: "facebook" },
      { title: "LinkedIn share link", sharer: "linkedin" },
      { title: "Email share link", sharer: "email" },
      { title: "Copy link", sharer: "" },
    ];

    // Verify each action exposes the current job URL and platform contract.
    for (const button of shareButtons) {
      const element = page.getByTitle(button.title);
      await expect(element).toBeVisible();
      if (button.title === "Copy link") {
        // Verify the copy action is available without an external destination.
        await expect(element).toBeEnabled();
        continue;
      }

      // Verify each external action receives the seeded public job URL.
      await expect.poll(async () => (await element.getAttribute("data-sharer")) || "").toBe(button.sharer);
      await expect.poll(async () => (await element.getAttribute("data-url")) || "").toContain("job_id=");
      const href = await element.getAttribute("href");
      expect(href).toBeTruthy();
      expect(href).toMatch(button.title === "Email share link" ? /^mailto:/ : /^https?:\/\//);
    }
  });
});
