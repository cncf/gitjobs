import { expect, test } from "../../../fixtures.js";
import { navigateWithRetries } from "../../../shared/navigation.js";
import { switchEmployerIfAvailable } from "../common/helpers.js";
import { openEmployerActionsDropdown } from "./helpers.js";

test.describe("GitJobs - Employer Jobs List", () => {
  test("closes employer job actions dropdown on Escape", async ({ employerPage: page }) => {
    // Load the employer jobs dashboard.
    await navigateWithRetries(page, "/dashboard/employer?tab=jobs", "employer jobs dashboard");

    // Open the first available job actions dropdown.
    const actionsDropdown = await openEmployerActionsDropdown(page);
    if (!actionsDropdown) {
      throw new Error("Expected at least one employer job action button.");
    }

    // Verify the actions dropdown exposes its open state.
    const { actionButton, dropdown } = actionsDropdown;
    await expect(dropdown).toBeVisible();
    await expect(actionButton).toHaveAttribute("aria-expanded", "true");
    await expect(dropdown).toHaveAttribute("aria-hidden", "false");

    // Close the actions dropdown with Escape.
    await page.keyboard.press("Escape");

    // Verify focus returns to the trigger and the dropdown is closed.
    await expect(dropdown).toBeHidden();
    await expect(actionButton).toBeFocused();
    await expect(actionButton).toHaveAttribute("aria-expanded", "false");
    await expect(dropdown).toHaveAttribute("aria-hidden", "true");
  });

  test("closes employer job actions dropdown after selecting an action", async ({ employerPage: page }) => {
    // Load the employer jobs dashboard.
    await navigateWithRetries(page, "/dashboard/employer?tab=jobs", "employer jobs dashboard");

    // Open the first available job actions dropdown.
    const actionsDropdown = await openEmployerActionsDropdown(page);
    if (!actionsDropdown) {
      throw new Error("Expected at least one employer job action button.");
    }

    // Verify the actions dropdown is visible.
    const { actionButton, dropdown } = actionsDropdown;
    await expect(dropdown).toBeVisible();

    // Find and activate the first available menu action.
    const menuAction = dropdown.locator('button[role="menuitem"]').first();
    await expect(menuAction).toBeVisible();
    await menuAction.click();

    // Verify action selection closes the dropdown.
    await expect(dropdown).toBeHidden();
    await expect(actionButton).toHaveAttribute("aria-expanded", "false");
    await expect(dropdown).toHaveAttribute("aria-hidden", "true");
  });

  test("closes employer job actions dropdown on outside click", async ({ employerPage: page }) => {
    // Load the employer jobs dashboard.
    await navigateWithRetries(page, "/dashboard/employer?tab=jobs", "employer jobs dashboard");

    // Open the first available job actions dropdown.
    const actionsDropdown = await openEmployerActionsDropdown(page);
    if (!actionsDropdown) {
      throw new Error("Expected at least one employer job action button.");
    }

    // Verify the actions dropdown exposes its open state.
    const { actionButton, dropdown } = actionsDropdown;
    await expect(dropdown).toBeVisible();
    await expect(actionButton).toHaveAttribute("aria-expanded", "true");
    await expect(dropdown).toHaveAttribute("aria-hidden", "false");

    // Dismiss the dropdown from outside its interactive region.
    await page
      .locator("#dashboard-content")
      .first()
      .click({
        position: { x: 8, y: 8 },
      });

    // Verify outside dismissal resets the dropdown state.
    await expect(dropdown).toBeHidden();
    await expect(actionButton).toHaveAttribute("aria-expanded", "false");
    await expect(dropdown).toHaveAttribute("aria-hidden", "true");
  });

  test("closes the previous job actions dropdown when opening another one", async ({
    employerPage: page,
  }) => {
    // Load the employer jobs dashboard.
    await navigateWithRetries(page, "/dashboard/employer?tab=jobs", "employer jobs dashboard");

    // Resolve an employer context with at least two job action buttons.
    let actionButtons = page.locator('.btn-actions[data-action-dropdown-bound="true"]:visible');
    let visibleButtonsCount = await actionButtons.count();
    for (let attempt = 0; attempt < 2 && visibleButtonsCount < 2; attempt++) {
      const switched = await switchEmployerIfAvailable(page);
      if (!switched) {
        break;
      }

      // Wait for the switched employer job actions to finish rendering.
      await expect
        .poll(
          async () => {
            return page.locator('.btn-actions[data-action-dropdown-bound="true"]:visible').count();
          },
          { timeout: 10000 },
        )
        .toBeGreaterThan(0);

      // Refresh the visible action-button collection after switching.
      actionButtons = page.locator('.btn-actions[data-action-dropdown-bound="true"]:visible');
      visibleButtonsCount = await actionButtons.count();
    }

    // Require two jobs before testing mutual exclusion between menus.
    if (visibleButtonsCount < 2) {
      throw new Error("Expected at least two employer job action buttons.");
    }

    // Open the first job actions dropdown.
    const firstActionsDropdown = await openEmployerActionsDropdown(page);
    if (!firstActionsDropdown) {
      throw new Error("Expected at least one employer job action button.");
    }

    // Read the first open menu contract.
    const { actionButton: firstButton, dropdown: firstDropdown, jobId: firstJobId } = firstActionsDropdown;

    // Find a second job actions trigger.
    const secondButton = page
      .locator(`.btn-actions[data-action-dropdown-bound="true"]:visible:not([data-job-id="${firstJobId}"])`)
      .first();
    await secondButton.waitFor({ state: "visible", timeout: 5000 });

    // Read and verify the second job identifier.
    const secondJobId = await secondButton.getAttribute("data-job-id");
    expect(secondJobId).toBeTruthy();

    // Find the second job actions dropdown.
    const secondDropdown = page.locator(`#dropdown-actions-${secondJobId}`);

    // Verify the first dropdown starts open.
    await expect(firstButton).toHaveAttribute("aria-expanded", "true");
    await expect(firstDropdown).toHaveAttribute("aria-hidden", "false");

    // Open the second dropdown through its bound client-side action.
    const clickedSecond = await page.evaluate((jobId) => {
      const button = document.querySelector(`#actions-btn-${jobId}`);
      if (!(button instanceof HTMLElement)) {
        return false;
      }

      // Activate the existing client-side dropdown binding.
      button.click();
      return true;
    }, secondJobId);
    expect(clickedSecond).toBe(true);

    // Verify the second dropdown reaches its open state.
    await expect
      .poll(async () => {
        const secondExpanded = await secondButton.getAttribute("aria-expanded");
        const secondHidden = await secondDropdown.getAttribute("aria-hidden");
        const secondClass = await secondDropdown.getAttribute("class");
        return secondExpanded === "true" && secondHidden === "false" && !secondClass?.includes("hidden");
      })
      .toBe(true);

    // Verify opening the second dropdown closes the first one.
    await expect(firstDropdown).toBeHidden();
    await expect(firstButton).toHaveAttribute("aria-expanded", "false");
    await expect(firstDropdown).toHaveAttribute("aria-hidden", "true");
  });
});
