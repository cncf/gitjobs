import { expect, test, TEST_USERS } from "../../fixtures.js";
import { resetJobApplications } from "../../shared/database.js";
import { navigateWithRetries, waitForActionResponse } from "../../shared/navigation.js";

const JOB_TITLE = "Frontend Developer";

test.describe("GitJobs - Application workflow", () => {
  test("applies to a job and cancels the application", async ({ applicantPage: page }) => {
    // Remove applications left by interrupted local runs.
    await resetJobApplications(TEST_USERS.applicant.userId);

    // Run the application lifecycle with guaranteed state restoration.
    try {
      // Load the job board filtered to the seeded published job.
      await navigateWithRetries(page, "/?ts_query=Frontend%20Developer", "filtered job board");

      // Find the seeded job card and read its identifier.
      const jobCard = page.locator('[data-preview-job="true"]', {
        hasText: JOB_TITLE,
      });
      const jobId = await jobCard.getAttribute("data-job-id");
      expect(jobId).toBeTruthy();
      // Open the job preview and verify its title.
      await jobCard.click();
      await expect(page.getByTestId("preview-job-title")).toHaveText(JOB_TITLE);

      // Apply to the job and confirm the action.
      await page.getByRole("button", { name: "Apply" }).click();
      await expect(page.getByRole("button", { name: "Yes" })).toBeVisible();
      await waitForActionResponse(page, () => page.getByRole("button", { name: "Yes" }).click(), {
        method: "POST",
        status: 204,
        urlEndsWith: `/jobs/${jobId}/apply`,
      });
      // Verify the application success feedback.
      await expect(page.getByText("You have successfully applied to this job!")).toBeVisible();

      // Load the job seeker applications dashboard.
      await navigateWithRetries(
        page,
        "/dashboard/job-seeker?tab=applications",
        "job seeker applications dashboard",
      );
      // Find the durable application and its cancellation control.
      const applicationRow = page.locator("#applications-list tr", {
        hasText: JOB_TITLE,
      });
      await expect(applicationRow).toBeVisible();
      const cancelButton = applicationRow.locator("[data-cancel-application-button]");
      const applicationId = (await cancelButton.getAttribute("id"))?.replace("cancel-application-", "");
      expect(applicationId).toBeTruthy();

      // Cancel the application and confirm the action.
      await cancelButton.click();
      await expect(page.getByRole("button", { name: "Yes" })).toBeVisible();
      await waitForActionResponse(page, () => page.getByRole("button", { name: "Yes" }).click(), {
        method: "PUT",
        status: 204,
        urlEndsWith: `/applications/${applicationId}/cancel`,
      });
      // Verify the cancelled application leaves the active list.
      await expect(applicationRow).toBeHidden();
    } finally {
      // Restore application state for later runs.
      await resetJobApplications(TEST_USERS.applicant.userId);
    }
  });
});
