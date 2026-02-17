import { test, expect } from "@playwright/test";
import {
  loginWithCredentials,
  openEmployerActionsDropdown,
  switchEmployerIfAvailable,
} from "../../shared/utils";
import { navigateHomeWithRetries } from "../../shared/helpers";

test.describe("GitJobs - Employer Jobs List", () => {
  test.beforeEach(async ({ page }) => {
    await navigateHomeWithRetries(page);
  });

  test("should close employer job actions dropdown on Escape", async ({
    page,
  }) => {
    await loginWithCredentials(page, "test", "test1234");
    await page.goto("/dashboard/employer?tab=jobs");

    const actionsDropdown = await openEmployerActionsDropdown(page);
    if (!actionsDropdown) {
      throw new Error("Expected at least one employer job action button.");
    }

    const { actionButton, dropdown } = actionsDropdown;
    await expect(dropdown).toBeVisible();
    await expect(actionButton).toHaveAttribute("aria-expanded", "true");
    await expect(dropdown).toHaveAttribute("aria-hidden", "false");

    await page.keyboard.press("Escape");

    await expect(dropdown).toBeHidden();
    await expect(actionButton).toBeFocused();
    await expect(actionButton).toHaveAttribute("aria-expanded", "false");
    await expect(dropdown).toHaveAttribute("aria-hidden", "true");
  });

  test("should close employer job actions dropdown after selecting an action", async ({
    page,
  }) => {
    await loginWithCredentials(page, "test", "test1234");
    await page.goto("/dashboard/employer?tab=jobs");

    const actionsDropdown = await openEmployerActionsDropdown(page);
    if (!actionsDropdown) {
      throw new Error("Expected at least one employer job action button.");
    }

    const { actionButton, dropdown } = actionsDropdown;
    await expect(dropdown).toBeVisible();

    const deleteAction = dropdown.locator("[data-delete-job-button]").first();
    await deleteAction.click();

    await expect(dropdown).toBeHidden();
    await expect(actionButton).toHaveAttribute("aria-expanded", "false");
    await expect(dropdown).toHaveAttribute("aria-hidden", "true");
  });

  test("should close employer job actions dropdown on outside click", async ({
    page,
  }) => {
    await loginWithCredentials(page, "test", "test1234");
    await page.goto("/dashboard/employer?tab=jobs");

    const actionsDropdown = await openEmployerActionsDropdown(page);
    if (!actionsDropdown) {
      throw new Error("Expected at least one employer job action button.");
    }

    const { actionButton, dropdown } = actionsDropdown;
    await expect(dropdown).toBeVisible();
    await expect(actionButton).toHaveAttribute("aria-expanded", "true");
    await expect(dropdown).toHaveAttribute("aria-hidden", "false");

    await page
      .locator("#dashboard-content")
      .first()
      .click({
        position: { x: 8, y: 8 },
      });

    await expect(dropdown).toBeHidden();
    await expect(actionButton).toHaveAttribute("aria-expanded", "false");
    await expect(dropdown).toHaveAttribute("aria-hidden", "true");
  });

  test("should close previously opened employer job actions dropdown when opening another one", async ({
    page,
  }) => {
    await loginWithCredentials(page, "test", "test1234");
    await page.goto("/dashboard/employer?tab=jobs");

    let actionButtons = page.locator(
      '.btn-actions[data-action-dropdown-bound="true"]:visible',
    );
    let visibleButtonsCount = await actionButtons.count();
    for (let attempt = 0; attempt < 2 && visibleButtonsCount < 2; attempt++) {
      const switched = await switchEmployerIfAvailable(page);
      if (!switched) {
        break;
      }

      await expect
        .poll(
          async () => {
            return page
              .locator('.btn-actions[data-action-dropdown-bound="true"]:visible')
              .count();
          },
          { timeout: 10000 },
        )
        .toBeGreaterThan(0);

      actionButtons = page.locator(
        '.btn-actions[data-action-dropdown-bound="true"]:visible',
      );
      visibleButtonsCount = await actionButtons.count();
    }

    if (visibleButtonsCount < 2) {
      throw new Error("Expected at least two employer job action buttons.");
    }

    const firstActionsDropdown = await openEmployerActionsDropdown(page);
    if (!firstActionsDropdown) {
      throw new Error("Expected at least one employer job action button.");
    }

    const {
      actionButton: firstButton,
      dropdown: firstDropdown,
      jobId: firstJobId,
    } = firstActionsDropdown;

    const secondButton = page
      .locator(
        `.btn-actions[data-action-dropdown-bound="true"]:visible:not([data-job-id="${firstJobId}"])`,
      )
      .first();
    await secondButton.waitFor({ state: "visible", timeout: 5000 });

    const secondJobId = await secondButton.getAttribute("data-job-id");
    expect(secondJobId).toBeTruthy();

    const secondDropdown = page.locator(`#dropdown-actions-${secondJobId}`);

    await expect(firstButton).toHaveAttribute("aria-expanded", "true");
    await expect(firstDropdown).toHaveAttribute("aria-hidden", "false");

    const clickedSecond = await page.evaluate((jobId) => {
      const button = document.querySelector(`#actions-btn-${jobId}`);
      if (!(button instanceof HTMLElement)) {
        return false;
      }

      button.click();
      return true;
    }, secondJobId);
    expect(clickedSecond).toBe(true);

    await expect
      .poll(async () => {
        const secondExpanded = await secondButton.getAttribute("aria-expanded");
        const secondHidden = await secondDropdown.getAttribute("aria-hidden");
        const secondClass = await secondDropdown.getAttribute("class");
        return (
          secondExpanded === "true" &&
          secondHidden === "false" &&
          !secondClass?.includes("hidden")
        );
      })
      .toBe(true);

    await expect(firstDropdown).toBeHidden();
    await expect(firstButton).toHaveAttribute("aria-expanded", "false");
    await expect(firstDropdown).toHaveAttribute("aria-hidden", "true");
  });
});
