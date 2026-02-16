import { Page, Locator } from '@playwright/test';

export const HOME_PATH = '/';
export const JOB_CARD_SELECTOR = '[data-preview-job="true"]';
export const JOB_TITLE_SELECTOR =
  '.text-base.font-stretch-condensed.font-medium.text-stone-900.line-clamp-2.md\\:line-clamp-1';

export const jobCards = (page: Page): Locator => page.locator(JOB_CARD_SELECTOR);

export const jobTitles = (page: Page): Locator => page.locator(JOB_TITLE_SELECTOR);

export const waitForJobCount = async (page: Page, expected: number): Promise<void> => {
  await page.waitForFunction(
    ({ selector, count }: { selector: string; count: number }) =>
      document.querySelectorAll(selector).length === count,
    { selector: JOB_CARD_SELECTOR, count: expected }
  );
};

export const jobTypeButtons = (page: Page): Locator =>
  page.getByRole('button', { name: /Job type/ });

export const searchInput = (page: Page): Locator =>
  page.locator('input[placeholder="Search jobs"]');

const userMenuButton = (page: Page): Locator =>
  page.locator('#user-dropdown-button:visible').first();

const userMenu = async (page: Page): Promise<Locator> => {
  const button = userMenuButton(page);
  await button.waitFor({ state: 'visible' });

  const controlledMenuId = await button.getAttribute('aria-controls');
  if (controlledMenuId) {
    return page.locator(`#${controlledMenuId}`).first();
  }

  return page.locator('#dropdown-user, #user-dropdown').first();
};

export const openUserMenu = async (page: Page): Promise<void> => {
  for (let attempt = 0; attempt < 2; attempt++) {
    const button = userMenuButton(page);
    const menu = await userMenu(page);
    if (await menu.isVisible()) {
      return;
    }

    await button.click();
    try {
      await menu.waitFor({ state: 'visible', timeout: 3000 });
      return;
    } catch {
      if (attempt === 1) {
        throw new Error('Failed to open user menu.');
      }
    }
  }
};

export const clickUserMenuItem = async (page: Page, label: string): Promise<void> => {
  let menu = await userMenu(page);
  if (!(await menu.isVisible())) {
    await openUserMenu(page);
    menu = await userMenu(page);
  }

  const menuItem = menu.getByRole('menuitem', { name: label, exact: true });
  if ((await menuItem.count()) > 0) {
    await menuItem.first().click();
    return;
  }

  const linkItem = menu.getByRole('link', { name: label, exact: true });
  if ((await linkItem.count()) > 0) {
    await linkItem.first().click();
    return;
  }

  await menu.locator(`a:has-text("${label}")`).first().click();
};

export const openEmployerActionsDropdown = async (
  page: Page
): Promise<{ actionButton: Locator; dropdown: Locator; jobId: string } | null> => {
  const actionButton = page.locator('.btn-actions:visible').first();

  try {
    await actionButton.waitFor({ state: 'visible', timeout: 5000 });
  } catch {
    return null;
  }

  const jobId = await actionButton.getAttribute('data-job-id');
  if (!jobId) {
    return null;
  }

  const dropdown = page.locator(`#dropdown-actions-${jobId}`);
  await actionButton.click();
  try {
    await dropdown.waitFor({ state: 'visible', timeout: 5000 });
  } catch {
    return null;
  }

  return { actionButton, dropdown, jobId };
};

export const openLoginPage = async (page: Page): Promise<void> => {
  await openUserMenu(page);
  await clickUserMenuItem(page, 'Log in');
  await page.waitForURL(/\/log-in(?:\?.*)?$/);
};

export const openSignUpPage = async (page: Page): Promise<void> => {
  await openUserMenu(page);
  await clickUserMenuItem(page, 'Sign up');
  await page.waitForURL(/\/sign-up(?:\?.*)?$/);
};

export const loginWithCredentials = async (
  page: Page,
  username: string,
  password: string
): Promise<void> => {
  await page.goto('/log-in');
  await page.locator('#username').fill(username);
  await page.locator('#password').fill(password);
  await page.getByRole('button', { name: 'Submit' }).click();
  await page.waitForURL((url) => url.pathname !== '/log-in');
};
