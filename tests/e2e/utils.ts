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

export const openUserMenu = async (page: Page): Promise<void> => {
  await page.locator('#user-dropdown-button').click();
};

export const openLoginPage = async (page: Page): Promise<void> => {
  await openUserMenu(page);
  await page.getByRole('link', { name: 'Log in' }).click();
  await page.waitForURL('**/log-in');
};

export const openSignUpPage = async (page: Page): Promise<void> => {
  await openUserMenu(page);
  await page.getByRole('link', { name: 'Sign up' }).click();
  await page.waitForURL('**/sign-up');
};

export const loginWithCredentials = async (
  page: Page,
  username: string,
  password: string
): Promise<void> => {
  await openLoginPage(page);
  await page.locator('#username').fill(username);
  await page.locator('#password').fill(password);
  await page.getByRole('button', { name: 'Submit' }).click();
};
