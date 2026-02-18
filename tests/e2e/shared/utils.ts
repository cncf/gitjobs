import { Page, Locator } from "@playwright/test";

export const HOME_PATH = "/";
export const JOB_CARD_SELECTOR = '[data-preview-job="true"]';
export const JOB_TITLE_SELECTOR = '[data-testid="job-card-title"]';

export const jobCards = (page: Page): Locator =>
  page.locator(JOB_CARD_SELECTOR);

export const jobTitles = (page: Page): Locator =>
  page.locator(JOB_TITLE_SELECTOR);

export const jobTypeButtons = (page: Page): Locator =>
  page.getByRole("button", { name: /Job type/ });

export const searchInput = (page: Page): Locator =>
  page.locator('input[placeholder="Search jobs"]');

const userMenuButton = (page: Page): Locator =>
  page.locator("#user-dropdown-button:visible").first();

const userMenu = async (page: Page): Promise<Locator> => {
  const button = userMenuButton(page);
  await button.waitFor({ state: "visible" });

  const controlledMenuId = await button.getAttribute("aria-controls");
  if (controlledMenuId) {
    return page.locator(`#${controlledMenuId}`).first();
  }

  return page.locator("#dropdown-user, #user-dropdown").first();
};

export const openUserMenu = async (page: Page): Promise<void> => {
  for (let attempt = 0; attempt < 3; attempt++) {
    const button = userMenuButton(page);
    const menu = await userMenu(page);
    if (await menu.isVisible()) {
      const hasVisibleItems =
        (await menu
          .locator('[role="menuitem"], a, button:not([disabled])')
          .filter({ visible: true })
          .count()) > 0;
      if (hasVisibleItems) {
        return;
      }
    }

    await button.click();
    try {
      await menu.waitFor({ state: "visible", timeout: 3000 });
      await menu
        .locator('[role="menuitem"], a, button:not([disabled])')
        .filter({ visible: true })
        .first()
        .waitFor({ state: "visible", timeout: 3000 });
      return;
    } catch {
      if (attempt === 2) {
        throw new Error("Failed to open user menu.");
      }
    }
  }
};

export const clickUserMenuItem = async (
  page: Page,
  label: string,
): Promise<void> => {
  let menu = await userMenu(page);
  if (!(await menu.isVisible())) {
    await openUserMenu(page);
    menu = await userMenu(page);
  }

  const visibleMenuItem = menu
    .getByRole("menuitem", { name: label, exact: true })
    .filter({ visible: true });
  if ((await visibleMenuItem.count()) > 0) {
    await visibleMenuItem.first().click();
    return;
  }

  const visibleLinkItem = menu
    .getByRole("link", { name: label, exact: true })
    .filter({ visible: true });
  if ((await visibleLinkItem.count()) > 0) {
    await visibleLinkItem.first().click();
    return;
  }

  await menu
    .locator(`a:has-text("${label}")`)
    .filter({ visible: true })
    .first()
    .click();
};

export const switchEmployerIfAvailable = async (
  page: Page,
): Promise<boolean> => {
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

  const selectableEmployers = dropdown.locator(
    "button.employer-button:not([disabled])",
  );
  if ((await selectableEmployers.count()) === 0) {
    return false;
  }

  await selectableEmployers.first().click();
  try {
    await page.waitForFunction(
      ({ previousLabel }) => {
        const button = document.querySelector("#employer-btn");
        return !!button && button.textContent?.trim() !== previousLabel;
      },
      { previousLabel: currentEmployerLabel },
      { timeout: 10000 },
    );
  } catch {
    return false;
  }
  await page
    .locator("#dashboard-content")
    .waitFor({ state: "visible", timeout: 10000 });

  return true;
};

export const openEmployerActionsDropdown = async (
  page: Page,
): Promise<{
  actionButton: Locator;
  dropdown: Locator;
  jobId: string;
} | null> => {
  let actionButton: Locator | null = null;
  for (let attempt = 0; attempt < 2; attempt++) {
    const candidate = page
      .locator('.btn-actions[data-action-dropdown-bound="true"]:visible')
      .first();

    try {
      await candidate.waitFor({ state: "visible", timeout: 5000 });
      actionButton = candidate;
      break;
    } catch {
      if (attempt === 1) {
        return null;
      }

      const switched = await switchEmployerIfAvailable(page);
      if (!switched) {
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
        const button = document.querySelector(
          `.btn-actions[data-job-id="${buttonId}"]`,
        );
        const menu = document.getElementById(dropdownId);
        if (!button || !menu) {
          return false;
        }

        const expanded = button.getAttribute("aria-expanded") === "true";
        const ariaVisible = menu.getAttribute("aria-hidden") === "false";
        const classVisible = !menu.classList.contains("hidden");
        return expanded && ariaVisible && classVisible;
      },
      { buttonId: jobId, dropdownId: `dropdown-actions-${jobId}` },
      { timeout: 10000 },
    );
  } catch {
    return null;
  }

  return { actionButton, dropdown, jobId };
};

export const openLoginPage = async (page: Page): Promise<void> => {
  await openUserMenu(page);
  await clickUserMenuItem(page, "Log in");
  await page.waitForURL(/\/log-in(?:\?.*)?$/);
};

export const openSignUpPage = async (page: Page): Promise<void> => {
  await openUserMenu(page);
  await clickUserMenuItem(page, "Sign up");
  await page.waitForURL(/\/sign-up(?:\?.*)?$/);
};

export const loginWithCredentials = async (
  page: Page,
  username: string,
  password: string,
): Promise<void> => {
  let loadedLoginPage = false;
  let lastError: unknown;
  for (let attempt = 0; attempt < 3; attempt++) {
    try {
      await page.goto("/log-in", { waitUntil: "domcontentloaded" });
      loadedLoginPage = true;
      break;
    } catch (error) {
      lastError = error;
    }
  }

  if (!loadedLoginPage) {
    throw new Error(
      `Failed to open log in page after 3 attempts: ${String(lastError)}`,
    );
  }

  await page.locator("#username").waitFor({ state: "visible", timeout: 10000 });
  await page.locator("#username").fill(username);
  await page.locator("#password").fill(password);
  await page.getByRole("button", { name: "Submit" }).click();
  await page.waitForURL((url) => url.pathname !== "/log-in", {
    timeout: 10000,
  });
};
