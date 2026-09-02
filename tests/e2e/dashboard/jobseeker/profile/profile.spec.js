import { expect, test, TEST_USERS } from "../../../fixtures.js";
import { resetJobSeekerProfile } from "../../../shared/database.js";
import { fillMarkdownEditor } from "../../../shared/forms.js";
import { navigateWithRetries, waitForActionResponse } from "../../../shared/navigation.js";

test.describe("GitJobs - Job Seeker Profile", () => {
  test("sends experience fields using bracket keys on profile update", async ({ jobSeekerPage: page }) => {
    try {
      // Load the job seeker profile form.
      await navigateWithRetries(page, "/dashboard/job-seeker", "job seeker dashboard");
      await page.locator("#name").waitFor({
        state: "visible",
        timeout: 10000,
      });

      // Fill the base profile fields through visible controls.
      await page.locator("#name").fill("E2E Job Seeker");
      await page.locator("#email").fill("jobseeker@t.com");
      await fillMarkdownEditor(page, "summary", "Profile summary");

      // Add experience through the real bracketed form fields.
      await page.locator('[data-section="experience"]').click();
      await page.locator('input[name="experience[0][title]"]').fill("Engineer");
      await page.locator('input[name="experience[0][company]"]').fill("ACME");
      await fillMarkdownEditor(page, "experience[0][description]", "Worked on platform");
      await page.locator('input[name="experience[0][start_date]"]').fill("2026-02-06");

      // Submit the profile and capture its update response.
      const updateResponse = await waitForActionResponse(
        page,
        () => page.locator("#update-profile-button").click(),
        {
          method: "PUT",
          urlEndsWith: "/dashboard/job-seeker/profile/update",
        },
      );

      // Parse the submitted form body.
      const body = updateResponse.request().postData() || "";
      const formData = new URLSearchParams(body);
      // Verify the submitted form contract and success feedback.
      expect(formData.get("experience[0][title]")).toBe("Engineer");
      expect(formData.get("experience[0][company]")).toBe("ACME");
      expect(formData.get("experience[0][description]")).toBe("Worked on platform");
      expect(formData.get("experience[0][start_date]")).toBe("2026-02-06");
      await expect(page.getByText("Profile updated successfully.")).toBeVisible();
    } finally {
      // Restore profile state for later runs.
      await resetJobSeekerProfile(TEST_USERS.jobSeeker.userId);
    }
  });
});
