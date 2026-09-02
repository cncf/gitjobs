import { navigateWithRetries, waitForActionResponse } from "./navigation.js";

/**
 * Clicks an item in the visible user menu.
 * @param {import("@playwright/test").Page} page - Page containing the menu.
 * @param {string} label - Visible item label.
 * @returns {Promise<void>}
 */
export const clickUserMenuItem = async (page, label) => {
  let menu = await userMenu(page);
  if (!(await menu.isVisible())) {
    await openUserMenu(page);
    menu = await userMenu(page);
  }

  const visibleMenuItem = menu.getByRole("menuitem", { name: label, exact: true }).filter({ visible: true });
  if ((await visibleMenuItem.count()) > 0) {
    await visibleMenuItem.first().click();
    return;
  }

  const visibleLinkItem = menu.getByRole("link", { name: label, exact: true }).filter({ visible: true });
  if ((await visibleLinkItem.count()) > 0) {
    await visibleLinkItem.first().click();
    return;
  }

  await menu.locator(`a:has-text("${label}")`).filter({ visible: true }).first().click();
};

/**
 * Logs in with a seeded E2E username and password.
 * @param {import("@playwright/test").Page} page - Page to authenticate.
 * @param {string} username - Seeded username.
 * @param {string} password - Seeded password.
 * @returns {Promise<void>}
 */
export const loginWithCredentials = async (page, username, password) => {
  await navigateWithRetries(page, "/log-in", "log in page");
  await page.locator("#username").waitFor({ state: "visible", timeout: 10000 });
  await page.locator("#username").fill(username);
  await page.locator("#password").fill(password);
  await waitForActionResponse(page, () => page.getByRole("button", { name: "Submit" }).click(), {
    method: "POST",
    status: 303,
    urlEndsWith: "/log-in",
  });
  await page.waitForURL((url) => url.pathname !== "/log-in", {
    timeout: 10000,
  });
};

/**
 * Opens the log-in page from the user menu.
 * @param {import("@playwright/test").Page} page - Page containing the menu.
 * @returns {Promise<void>}
 */
export const openLoginPage = async (page) => {
  await openUserMenu(page);
  await clickUserMenuItem(page, "Log in");
  await page.waitForURL(/\/log-in(?:\?.*)?$/);
};

/**
 * Opens the sign-up page from the user menu.
 * @param {import("@playwright/test").Page} page - Page containing the menu.
 * @returns {Promise<void>}
 */
export const openSignUpPage = async (page) => {
  await openUserMenu(page);
  await clickUserMenuItem(page, "Sign up");
  await page.waitForURL(/\/sign-up(?:\?.*)?$/);
};

/**
 * Opens the user menu after its client-side items are ready.
 * @param {import("@playwright/test").Page} page - Page containing the menu.
 * @returns {Promise<void>}
 */
export const openUserMenu = async (page) => {
  for (let attempt = 0; attempt < 3; attempt++) {
    const button = userMenuButton(page);
    const menu = await userMenu(page);
    if (await hasVisibleMenuItems(menu)) {
      return;
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

/**
 * Returns whether a menu contains visible interactive items.
 * @param {import("@playwright/test").Locator} menu - User menu locator.
 * @returns {Promise<boolean>}
 */
const hasVisibleMenuItems = async (menu) => {
  return (
    (await menu.locator('[role="menuitem"], a, button:not([disabled])').filter({ visible: true }).count()) > 0
  );
};

/**
 * Resolves the user menu controlled by the visible header button.
 * @param {import("@playwright/test").Page} page - Page containing the menu.
 * @returns {Promise<import("@playwright/test").Locator>}
 */
const userMenu = async (page) => {
  const button = userMenuButton(page);
  await button.waitFor({ state: "visible" });

  const controlledMenuId = await button.getAttribute("aria-controls");
  if (controlledMenuId) {
    return page.locator(`#${controlledMenuId}`).first();
  }

  return page.locator("#dropdown-user, #user-dropdown").first();
};

/**
 * Returns the visible user menu button.
 * @param {import("@playwright/test").Page} page - Page containing the button.
 * @returns {import("@playwright/test").Locator}
 */
const userMenuButton = (page) => {
  return page.locator("#user-dropdown-button:visible").first();
};
