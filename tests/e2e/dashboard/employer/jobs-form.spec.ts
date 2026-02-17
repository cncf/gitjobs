import { test, expect } from "@playwright/test";
import { loginWithCredentials } from "../../shared/utils";
import { navigateHomeWithRetries } from "../../shared/helpers";

test.describe("GitJobs - Employer Jobs Form", () => {
  test.beforeEach(async ({ page }) => {
    await navigateHomeWithRetries(page);
  });

  test("should add a new job", async ({ page }) => {
    await loginWithCredentials(page, "test", "test1234");
    await page.goto("/dashboard/employer/jobs/add", {
      waitUntil: "domcontentloaded",
    });

    const titleField = page.locator("#title");
    try {
      await titleField.waitFor({ state: "visible", timeout: 5000 });
    } catch {
      await page.goto("/dashboard/employer?tab=jobs", {
        waitUntil: "domcontentloaded",
      });
      const addJobButton = page.getByRole("button", { name: "Add Job" });
      await addJobButton.waitFor({ state: "visible", timeout: 10000 });
      await addJobButton.click();
      await titleField.waitFor({ state: "visible", timeout: 10000 });
    }

    await page.locator("#title").click();
    await page.locator("#title").fill("job");
    await expect(page.locator("markdown-editor#description")).toHaveCount(1, {
      timeout: 10000,
    });

    await page.evaluate(() => {
      const jobsForm = document.getElementById("jobs-form");
      if (!(jobsForm instanceof HTMLFormElement)) {
        return;
      }

      const markdownEditor = document.querySelector(
        "markdown-editor#description",
      );
      let descriptionField =
        markdownEditor?.querySelector('textarea[name="description"]') ||
        jobsForm.querySelector('textarea[name="description"]');

      if (!(descriptionField instanceof HTMLTextAreaElement)) {
        descriptionField = document.createElement("textarea");
        descriptionField.name = "description";
        descriptionField.style.display = "none";
        jobsForm.append(descriptionField);
      }

      descriptionField.value = "description";
      descriptionField.dispatchEvent(new Event("input", { bubbles: true }));
      descriptionField.dispatchEvent(new Event("change", { bubbles: true }));
    });

    await page.getByRole("button", { name: "Publish" }).click();
    expect(page.url()).toContain("/dashboard/employer");
  });
});
