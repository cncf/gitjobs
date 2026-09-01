import { expect, test } from "../../../fixtures.js";
import { resetModeratedJob } from "../../../shared/database.js";
import { navigateWithRetries, waitForActionResponse } from "../../../shared/navigation.js";

const APPROVAL_JOB_ID = "22222222-2222-4222-8222-222222222201";
const APPROVAL_JOB_TITLE = "E2E job awaiting approval";
const REJECTION_JOB_ID = "22222222-2222-4222-8222-222222222202";
const REJECTION_JOB_TITLE = "E2E job awaiting rejection";

test.describe("GitJobs - Moderator Jobs", () => {
  test("approves a pending job", async ({ moderatorPage: page }) => {
    // Restore the seeded job to its pending moderation state.
    await resetModeratedJob(APPROVAL_JOB_ID);

    // Run approval checks with guaranteed state restoration.
    try {
      // Load the pending moderation table.
      await navigateWithRetries(
        page,
        "/dashboard/moderator?tab=pending-jobs",
        "pending moderator jobs dashboard",
      );
      // Find the seeded pending job.
      const pendingJobRow = page.locator("tr", {
        hasText: APPROVAL_JOB_TITLE,
      });
      await expect(pendingJobRow).toBeVisible();
      // Approve the job and verify the row leaves the pending table.
      await waitForActionResponse(page, () => pendingJobRow.getByTitle("Approve").click(), {
        method: "PUT",
        status: 204,
        urlEndsWith: `/jobs/${APPROVAL_JOB_ID}/approve`,
      });
      await expect(pendingJobRow).toBeHidden();

      // Verify the approved job is durable in the live moderation table.
      await navigateWithRetries(page, "/dashboard/moderator?tab=live-jobs", "live moderator jobs dashboard");
      await expect(page.locator("tr", { hasText: APPROVAL_JOB_TITLE })).toBeVisible();
    } finally {
      // Restore the reusable pending moderation state.
      await resetModeratedJob(APPROVAL_JOB_ID);
    }
  });

  test("rejects a pending job with review notes", async ({ moderatorPage: page }) => {
    // Restore the seeded job to its pending moderation state.
    await resetModeratedJob(REJECTION_JOB_ID);

    // Run rejection checks with guaranteed state restoration.
    try {
      // Load the pending moderation table.
      await navigateWithRetries(
        page,
        "/dashboard/moderator?tab=pending-jobs",
        "pending moderator jobs dashboard",
      );
      // Find the seeded pending job.
      const pendingJobRow = page.locator("tr", {
        hasText: REJECTION_JOB_TITLE,
      });
      await expect(pendingJobRow).toBeVisible();
      // Open the rejection modal and provide review notes.
      await pendingJobRow.getByTitle("Reject").click();
      const rejectModal = page.locator("#reject-modal");
      await expect(rejectModal).toBeVisible();
      await rejectModal.getByLabel("Review notes").fill("The description needs more detail.");

      // Submit the review and verify the pending row is removed.
      await waitForActionResponse(page, () => rejectModal.getByRole("button", { name: "Reject" }).click(), {
        method: "PUT",
        status: 204,
        urlEndsWith: `/jobs/${REJECTION_JOB_ID}/reject`,
      });
      await expect(rejectModal).toBeHidden();
      await expect(pendingJobRow).toBeHidden();
    } finally {
      // Restore the reusable pending moderation state.
      await resetModeratedJob(REJECTION_JOB_ID);
    }
  });

  test("closes moderator mobile menu drawer on Escape @mobile", async ({ moderatorPage: page }) => {
    // Load the moderator dashboard using the mobile project.
    await navigateWithRetries(page, "/dashboard/moderator", "moderator dashboard");

    // Find the mobile menu trigger.
    const openMenuButton = page.locator("#open-menu-button");
    await expect(openMenuButton).toBeVisible();

    // Open the mobile moderator menu.
    await openMenuButton.click();

    // Find the drawer and verify its accessible open state.
    const drawer = page.locator("#drawer-menu");
    await expect(drawer).toHaveAttribute("data-open", "true");
    await expect(drawer).toHaveAttribute("aria-hidden", "false");

    // Close the drawer with Escape.
    await page.keyboard.press("Escape");

    // Verify the drawer exposes its accessible closed state.
    await expect(drawer).toHaveAttribute("data-open", "false");
    await expect(drawer).toHaveAttribute("aria-hidden", "true");
  });
});
