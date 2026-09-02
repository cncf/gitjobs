import { buildE2eUrl } from "../../../shared/navigation.js";
import { switchEmployerIfAvailable } from "../common/helpers.js";

/**
 * @typedef {object} EmployerActionsDropdown
 * @property {import("@playwright/test").Locator} actionButton - Menu button.
 * @property {import("@playwright/test").Locator} dropdown - Actions menu.
 * @property {string} jobId - Job identifier.
 */

/**
 * Deletes a job created by the current authenticated employer test page.
 * @param {import("@playwright/test").Page} page - Authenticated employer page.
 * @param {string} jobId - Job identifier.
 * @returns {Promise<void>}
 */
export const deleteEmployerJob = async (page, jobId) => {
  const response = await page.request.delete(buildE2eUrl(employerJobActionPath(jobId, "delete")), {
    headers: { "HX-Request": "true" },
  });
  if (!response.ok()) {
    throw new Error(`Failed to delete E2E job ${jobId}: received ${response.status()}.`);
  }
};

/**
 * Opens an available action menu for a job owned by the selected employer.
 * @param {import("@playwright/test").Page} page - Authenticated employer page.
 * @returns {Promise<EmployerActionsDropdown | null>}
 */
export const openEmployerActionsDropdown = async (page) => {
  let actionButton = null;
  for (let attempt = 0; attempt < 2; attempt++) {
    const candidate = page.locator('.btn-actions[data-action-dropdown-bound="true"]:visible').first();

    try {
      await candidate.waitFor({ state: "visible", timeout: 5000 });
      actionButton = candidate;
      break;
    } catch {
      if (attempt === 1 || !(await switchEmployerIfAvailable(page))) {
        return null;
      }
    }
  }

  if (!actionButton) {
    return null;
  }

  const jobId = await actionButton.getAttribute("data-job-id");
  if (!jobId) {
    return null;
  }

  const dropdown = page.locator(`#dropdown-actions-${jobId}`);

  await actionButton.scrollIntoViewIfNeeded();
  await actionButton.click({ force: true });
  try {
    await page.waitForFunction(
      ({ buttonId, dropdownId }) => {
        const button = document.querySelector(`.btn-actions[data-job-id="${buttonId}"]`);
        const menu = document.getElementById(dropdownId);
        if (!button || !menu) {
          return false;
        }

        return (
          button.getAttribute("aria-expanded") === "true" &&
          menu.getAttribute("aria-hidden") === "false" &&
          !menu.classList.contains("hidden")
        );
      },
      { buttonId: jobId, dropdownId: `dropdown-actions-${jobId}` },
      { timeout: 10000 },
    );
  } catch {
    return null;
  }

  return { actionButton, dropdown, jobId };
};

/**
 * Builds an employer job action path.
 * @param {string} jobId - Job identifier.
 * @param {string} action - Route action.
 * @returns {string}
 */
const employerJobActionPath = (jobId, action) => {
  return `/dashboard/employer/jobs/${jobId}/${action}`;
};
