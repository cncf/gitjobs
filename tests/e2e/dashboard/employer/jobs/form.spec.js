import { expect, test } from "../../../fixtures.js";
import { deleteJobsByTitlePrefix } from "../../../shared/database.js";
import { fillMarkdownEditor } from "../../../shared/forms.js";
import { navigateWithRetries, waitForActionResponse } from "../../../shared/navigation.js";
import { deleteEmployerJob } from "./helpers.js";

const JOB_TITLE_PREFIX = "E2E lifecycle job";

test.describe("GitJobs - Employer Jobs Form", () => {
  test("adds a new job", async ({ employerPage: page }) => {
    // Clear jobs left by interrupted local runs.
    await deleteJobsByTitlePrefix(JOB_TITLE_PREFIX);

    // Load the dashboard shell that registers its form components.
    await navigateWithRetries(page, "/dashboard/employer?tab=jobs", "employer jobs dashboard");

    // Open the add-job fragment through its dashboard action.
    const addJobButton = page.getByRole("button", { name: "Add Job" });
    await expect(addJobButton).toBeVisible();
    await waitForActionResponse(page, () => addJobButton.click(), {
      method: "GET",
      urlEndsWith: "/dashboard/employer/jobs/add",
    });

    // Verify the fragment and its markdown editor finish mounting.
    await expect(page.locator("#title")).toBeVisible();
    await expect(page.locator('markdown-editor textarea[name="description"]')).toBeAttached();

    // Prepare a unique job title for durable verification and cleanup.
    const jobTitle = `${JOB_TITLE_PREFIX} ${Date.now()}`;
    let createdJobId = null;

    // Run creation checks with guaranteed job cleanup.
    try {
      // Create a pending job through the real markdown editor and form action.
      await page.locator("#title").fill(jobTitle);
      await fillMarkdownEditor(page, "description", "E2E job description");

      // Publish the job and verify the creation response.
      await waitForActionResponse(page, () => page.getByRole("button", { name: "Publish" }).click(), {
        method: "POST",
        urlEndsWith: "/dashboard/employer/jobs/add",
      });

      // Verify the creation success feedback.
      await expect(page.getByText("Job added successfully.")).toBeVisible();

      // Load the employer jobs table and find the durable job row.
      await navigateWithRetries(page, "/dashboard/employer?tab=jobs", "employer jobs dashboard");
      const createdJobRow = page.locator("tr", { hasText: jobTitle });
      await expect(createdJobRow).toBeVisible();
      createdJobId = await createdJobRow.locator("[data-job-id]").first().getAttribute("data-job-id");
      expect(createdJobId).toBeTruthy();
    } finally {
      // Restore durable state even when a UI assertion fails before the id is captured.
      try {
        if (createdJobId) {
          await deleteEmployerJob(page, createdJobId);
        }
      } finally {
        await deleteJobsByTitlePrefix(JOB_TITLE_PREFIX);
      }
    }
  });
});
