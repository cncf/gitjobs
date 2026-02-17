import { test, expect } from "@playwright/test";
import { loginWithCredentials } from "../../shared/utils";
import { navigateHomeWithRetries } from "../../shared/helpers";

test.describe("GitJobs - Job Seeker Profile", () => {
  test.beforeEach(async ({ page }) => {
    await navigateHomeWithRetries(page);
  });

  test("should send experience fields using bracket keys on profile update", async ({
    page,
  }) => {
    const setMarkdownValue = async (
      selector: string,
      value: string,
    ): Promise<void> => {
      const textarea = page.locator(selector).first();
      await textarea.waitFor({ state: "attached", timeout: 10000 });
      await textarea.evaluate((element, text) => {
        if (!(element instanceof HTMLTextAreaElement)) {
          return;
        }

        element.value = text;
        element.dispatchEvent(new Event("input", { bubbles: true }));
        element.dispatchEvent(new Event("change", { bubbles: true }));
      }, value);
    };

    await loginWithCredentials(page, "test", "test1234");
    await page.goto("/dashboard/job-seeker", { waitUntil: "domcontentloaded" });
    await page.locator("#name").waitFor({ state: "visible", timeout: 10000 });

    await page.locator("#name").fill("Test User");
    await page.locator("#email").fill("test@example.com");
    await setMarkdownValue('textarea[name="summary"]', "Profile summary");

    await page.locator('[data-section="experience"]').click();
    await page.locator('input[name="experience[0][title]"]').fill("Engineer");
    await page.locator('input[name="experience[0][company]"]').fill("ACME");
    await setMarkdownValue(
      'textarea[name="experience[0][description]"]',
      "Worked on platform",
    );
    await page
      .locator('input[name="experience[0][start_date]"]')
      .fill("2026-02-06");

    const requestPromise = page.waitForRequest((request) => {
      return (
        request.method() === "PUT" &&
        request.url().includes("/dashboard/job-seeker/profile/update")
      );
    });

    await page.locator("#update-profile-button").click();

    const updateRequest = await requestPromise;
    const body = updateRequest.postData() || "";
    const formData = new URLSearchParams(body);
    expect(formData.get("experience[0][title]")).toBe("Engineer");
    expect(formData.get("experience[0][company]")).toBe("ACME");
    expect(formData.get("experience[0][description]")).toBe(
      "Worked on platform",
    );
    expect(formData.get("experience[0][start_date]")).toBe("2026-02-06");
  });
});
