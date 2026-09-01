import { expect, test } from "@playwright/test";

import { navigateHomeWithRetries } from "../../shared/navigation.js";
import { jobCards } from "./helpers.js";

test.describe("GitJobs - Jobboard smoke", () => {
  test("renders seeded job results", async ({ page }) => {
    // Load the public job board.
    await navigateHomeWithRetries(page);

    // Verify its shell and seeded result list are ready.
    await expect(page).toHaveTitle(/GitJobs/);
    await expect(page.getByRole("heading", { level: 1 })).toBeVisible();
    await expect(jobCards(page).first()).toBeVisible();
  });
});
