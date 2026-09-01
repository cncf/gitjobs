import { buildE2eUrl } from "../../../shared/navigation.js";

/**
 * Selects the employer context used by authenticated employer fixtures.
 * @param {import("@playwright/test").Page} page - Authenticated employer page.
 * @param {string} employerId - Seeded employer identifier.
 * @returns {Promise<void>}
 */
export const selectEmployerContext = async (page, employerId) => {
  const response = await page.request.put(buildE2eUrl(`/dashboard/employer/employers/${employerId}/select`), {
    headers: { "HX-Request": "true" },
  });
  if (response.status() !== 204) {
    throw new Error(`Failed to select E2E employer ${employerId}: received ${response.status()}.`);
  }
};

/**
 * Switches to another seeded employer when one is available.
 * @param {import("@playwright/test").Page} page - Authenticated employer page.
 * @returns {Promise<boolean>}
 */
export const switchEmployerIfAvailable = async (page) => {
  const employerButton = page.locator("#employer-btn");
  if ((await employerButton.count()) === 0) {
    return false;
  }
  const currentEmployerLabel = (await employerButton.innerText()).trim();

  const dropdown = page.locator("#dropdown-employers");
  await employerButton.click();
  try {
    await dropdown.waitFor({ state: "visible", timeout: 5000 });
  } catch {
    return false;
  }

  const selectableEmployers = dropdown.locator("button.employer-button:not([disabled])");
  if ((await selectableEmployers.count()) === 0) {
    return false;
  }

  await selectableEmployers.first().click();
  try {
    await page.waitForFunction(
      ({ previousLabel }) => {
        const button = document.querySelector("#employer-btn");
        return Boolean(button && button.textContent?.trim() !== previousLabel);
      },
      { previousLabel: currentEmployerLabel },
      { timeout: 10000 },
    );
  } catch {
    return false;
  }
  await page.locator("#dashboard-content").waitFor({ state: "visible", timeout: 10000 });

  return true;
};
