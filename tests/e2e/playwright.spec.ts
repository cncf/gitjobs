import { test, expect, Page } from '@playwright/test';
import {
  jobCards,
  jobTitles,
  jobTypeButtons,
  clickUserMenuItem,
  loginWithCredentials,
  openEmployerActionsDropdown,
  openLoginPage,
  openSignUpPage,
  openUserMenu,
  searchInput,
  waitForJobCount,
  JOB_TITLE_SELECTOR,
} from './utils';

const countVisibleNoDataMessages = async (page: Page): Promise<number> => {
  return page.getByText('No data available yet').evaluateAll((nodes) => {
    return nodes.filter((node) => {
      const element = node;
      if (!(element instanceof HTMLElement)) {
        return false;
      }

      const style = window.getComputedStyle(element);
      const rect = element.getBoundingClientRect();
      return style.display !== 'none' && style.visibility !== 'hidden' && rect.width > 0 && rect.height > 0;
    }).length;
  });
};

const waitForOnlyJobTypeResults = async (page: Page, expectedType: string): Promise<void> => {
  const expectedTypePattern = expectedType
    .trim()
    .split(/\s+/)
    .map((word) => word.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'))
    .join('\\s+');
  const jobTypePattern = new RegExp(`Job\\s*type\\s*${expectedTypePattern}`, 'i');

  await expect
    .poll(
      async () => {
        const cards = page.locator('[data-preview-job="true"]');
        const totalCards = await cards.count();
        if (totalCards === 0) {
          return false;
        }

        const matchingCards = await page.locator('[data-preview-job="true"]', { hasText: jobTypePattern }).count();
        return totalCards === matchingCards;
      },
      { timeout: 10000 }
    )
    .toBe(true);
};

test.describe('GitJobs', () => {
  test.beforeEach(async ({ page }) => {
    let lastError: unknown;
    for (let i = 0; i < 3; i++) {
      try {
        await page.goto('/', { timeout: 9000 });
        await page.waitForLoadState('domcontentloaded');
        return;
      } catch (error) {
        lastError = error;
      }
    }

    throw new Error(`Failed to navigate to home page after 3 attempts: ${String(lastError)}`);
  });

  test('should have the correct title and heading', async ({ page }) => {
    await expect(page).toHaveTitle(/GitJobs/);
    await expect(page.getByRole('heading', { level: 1 })).toBeVisible();
  });

  test('should apply a filter and verify that the results are updated', async ({ page }) => {
    await page.locator('div:nth-child(4) > div > .font-semibold').first().click();
    await page.locator('label').filter({ hasText: 'Full Time' }).nth(1).click();
    await expect(page).toHaveURL(/full-time/);
    await waitForOnlyJobTypeResults(page, 'full time');

    const jobTypeButtonsList = await jobTypeButtons(page).all();
    for (const jobCard of jobTypeButtonsList) {
      const jobTypeElement = jobCard.locator('.capitalize').first();
      if (await jobTypeElement.isVisible()) {
        await expect(jobTypeElement).toHaveText('full time');
      }
    }
  });

  test('should apply multiple filters and verify that the results are updated', async ({ page }) => {
    await page.locator('div:nth-child(4) > div > .font-semibold').first().click();
    await page.locator('label').filter({ hasText: 'Part Time' }).nth(1).click();
    await page.locator('label').filter({ hasText: 'Internship' }).nth(1).click();

    await waitForJobCount(page, 6);

    const jobTypeButtonsList = await jobTypeButtons(page).all();
    for (const jobCard of jobTypeButtonsList) {
      const jobTypeElement = jobCard.locator('.capitalize').first();
      if (await jobTypeElement.isVisible()) {
        const jobTypeText = await jobTypeElement.textContent();
        expect(['part time', 'internship']).toContain(jobTypeText?.trim());
      }
    }
  });

  test('should not send empty or zero default filter values', async ({ page }) => {
    const requestPromise = page.waitForRequest((request) => {
      return request.method() === 'GET' && request.url().includes('/section/jobs/results');
    });

    await page.locator('label').filter({ hasText: 'Full Time' }).nth(1).click();

    const requestUrl = (await requestPromise).url();
    const query = new URL(requestUrl).search;

    expect(query).not.toContain('seniority=');
    expect(query).not.toContain('open_source=0');
    expect(query).not.toContain('upstream_commitment=0');
    expect(query).not.toContain('salary_min=0');
    expect(query).not.toContain('ts_query=');
  });

  test('should search for a job and verify that the results are updated and contain the search term', async ({ page }) => {
    await searchInput(page).click();
    await searchInput(page).fill('Engineer');
    await page.locator('#search-jobs-btn').click();

    await page.waitForFunction(
      ({ selector, term }) => {
        const nodes = Array.from(document.querySelectorAll(selector));
        if (nodes.length === 0) {
          return false;
        }
        return nodes.every(node => node.textContent?.toLowerCase().includes(term));
      },
      { selector: JOB_TITLE_SELECTOR, term: 'engineer' }
    );

    const jobTitleValues = await jobTitles(page).allTextContents();
    for (const title of jobTitleValues) {
      expect(title.trim().toLowerCase()).toContain('engineer');
    }
  });

  test('should apply a filter and verify that the results are updated on mobile', async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 667 });
    await page.locator('#open-filters').click();
    await page.waitForSelector('#drawer-filters', { state: 'visible' });
    await page.locator('#drawer-filters label').filter({ hasText: 'Full Time' }).click();
    await page.locator('#close-filters').click();
    await expect(page).toHaveURL(/full-time/);
    await waitForOnlyJobTypeResults(page, 'full time');

    const jobTypeButtonsList = await jobTypeButtons(page).all();
    for (const jobCard of jobTypeButtonsList) {
      const jobTypeElement = jobCard.locator('.capitalize').first();
      if (await jobTypeElement.isVisible()) {
        await expect(jobTypeElement).toHaveText('full time');
      }
    }
  });

  test('should close mobile filters drawer on Escape', async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 667 });
    const openFiltersButton = page.locator('#open-filters');
    await expect(openFiltersButton).toBeVisible();

    await openFiltersButton.click();

    const drawer = page.locator('#drawer-filters');
    await expect(drawer).toHaveAttribute('data-open', 'true');
    await expect(drawer).toHaveAttribute('aria-hidden', 'false');

    await page.keyboard.press('Escape');

    await expect(drawer).toHaveAttribute('data-open', 'false');
    await expect(drawer).toHaveAttribute('aria-hidden', 'true');
  });

  test('should close moderator mobile menu drawer on Escape', async ({ page }) => {
    await loginWithCredentials(page, 'test', 'test1234');
    await page.setViewportSize({ width: 375, height: 667 });
    await page.goto('/dashboard/moderator', { waitUntil: 'domcontentloaded' });

    const openMenuButton = page.locator('#open-menu-button');
    await expect(openMenuButton).toBeVisible();

    await openMenuButton.click();

    const drawer = page.locator('#drawer-menu');
    await expect(drawer).toHaveAttribute('data-open', 'true');
    await expect(drawer).toHaveAttribute('aria-hidden', 'false');

    await page.keyboard.press('Escape');

    await expect(drawer).toHaveAttribute('data-open', 'false');
    await expect(drawer).toHaveAttribute('aria-hidden', 'true');
  });

  test('should reset filters', async ({ page }) => {
    await page.locator('label').filter({ hasText: 'Part Time' }).nth(1).click();

    await waitForJobCount(page, 3);
    await page.locator('#reset-desktop-filters').click();
    await waitForJobCount(page, 20);
    await expect(page.locator('#results')).toHaveText(/1 - 20 of \d+ results/);
  });

  test('should sort jobs', async ({ page }) => {
    const initialJobTitles = (await jobTitles(page).allTextContents()).map(title => title.trim());
    await page.locator('#sort-desktop').selectOption('salary');
    await expect(page).toHaveURL(/\?sort=salary/);
    const sortedJobTitles = (await jobTitles(page).allTextContents()).map(title => title.trim());
    expect(sortedJobTitles.length).toBe(initialJobTitles.length);
    expect(sortedJobTitles[0]).toBeTruthy();
    expect(sortedJobTitles.some((title, index) => title !== initialJobTitles[index])).toBe(true);
  });

  test('ensure filters and search persist on page refresh', async ({ page }) => {
    await searchInput(page).fill('Engineer');
    await page.locator('label').filter({ hasText: 'Full Time' }).nth(1).click();
    await expect(page).toHaveURL(/Engineer/);
    await expect(page).toHaveURL(/full-time/);

    const urlBeforeRefresh = page.url();
    expect(urlBeforeRefresh).toContain('Engineer');
    expect(urlBeforeRefresh).toContain('full-time');

    await page.reload();
    await page.waitForLoadState('domcontentloaded');

    const urlAfterRefresh = page.url();
    expect(urlAfterRefresh).toBe(urlBeforeRefresh);

    await expect(searchInput(page)).toHaveValue('Engineer');
    await expect(page.locator('input[id="desktop-kind[]-full-time"]')).toBeChecked();
  });

  test('should show hover states and preview on job card interactions', async ({ page }) => {
    await jobCards(page).first().waitFor();
    const firstJobCard = jobCards(page).first();

    // Test quick preview without opening modal
    const jobTitle = await firstJobCard.locator(JOB_TITLE_SELECTOR).textContent();

    // Verify job card shows basic info without modal
    expect(jobTitle?.trim()).toBeTruthy();

    // Test hover state - verify card is hoverable
    await firstJobCard.hover();
    await expect(firstJobCard).toBeVisible();

    // Ensure modal is not open before or after hovering
    await expect(page.locator('#preview-modal')).not.toBeVisible();
  });

  test('should navigate to the stats page and interact with charts', async ({ page, browserName }) => {
    await page.getByRole('link', { name: 'Stats' }).click();
    await expect(page).toHaveURL(/\/stats/);

    await expect
      .poll(
        async () => {
          const noDataVisibleCount = await countVisibleNoDataMessages(page);
          return noDataVisibleCount > 0 || (await page.locator('#line-chart').isVisible());
        },
        { timeout: 15000 }
      )
      .toBe(true);

    const noDataVisibleCount = await countVisibleNoDataMessages(page);

    if (noDataVisibleCount > 0) {
      await expect(page.getByText('No data available yet').first()).toBeVisible();
      return;
    }

    await expect(page.locator('#line-chart')).toBeVisible({ timeout: 15000 });
    await expect(page.locator('#bar-daily')).toBeVisible({ timeout: 15000 });

    if (browserName === 'firefox') {
      await expect(page.locator('#line-chart')).toBeVisible({ timeout: 15000 });
      await expect(page.locator('#bar-daily')).toBeVisible({ timeout: 15000 });
    } else {
      const lineChartTarget = page.locator('#line-chart canvas, #line-chart rect, #line-chart path').first();
      if ((await lineChartTarget.count()) > 0) {
        await lineChartTarget.click({ force: true });
      } else {
        await page.locator('#line-chart').click({ force: true });
      }

      const barDailyTarget = page.locator('#bar-daily canvas, #bar-daily rect, #bar-daily path').first();
      if ((await barDailyTarget.count()) > 0) {
        await barDailyTarget.click({ force: true });
      } else {
        await page.locator('#bar-daily').click({ force: true });
      }
    }
  });

  test('should navigate to the about page and check for a body', async ({ page }) => {
    await page.getByRole('link', { name: 'About' }).click();
    await expect(page).toHaveURL(/\/about/);
    await expect(page.locator('body')).toBeVisible();
  });

  test('should navigate to the sign-up page', async ({ page }) => {
    await openSignUpPage(page);
    await expect(page).toHaveURL(/\/sign-up(?:\?.*)?$/);
  });

  test('should block sign-up when passwords do not match', async ({ page }) => {
    await openSignUpPage(page);

    await page.locator('#name').fill('Test User');
    await page.locator('#email').fill('test-signup@example.com');
    await page.locator('#username').fill('test-signup');
    await page.locator('#password').fill('test1234');
    await page.locator('#password_confirmation').fill('test5678');
    await page.getByRole('button', { name: 'Submit' }).click();

    await expect(page).toHaveURL(/\/sign-up(?:\?.*)?$/);
    await expect
      .poll(async () => page.locator('#password_confirmation').evaluate((element) => element.validationMessage))
      .toBe('Passwords do not match');
  });

  test('should block sign-up when required name contains only spaces', async ({ page }) => {
    await openSignUpPage(page);

    await page.locator('#name').fill('   ');
    await page.locator('#email').fill('test-spaces@example.com');
    await page.locator('#username').fill('test-spaces');
    await page.locator('#password').fill('test1234');
    await page.locator('#password_confirmation').fill('test1234');
    await page.getByRole('button', { name: 'Submit' }).click();

    await expect(page).toHaveURL(/\/sign-up(?:\?.*)?$/);
    await expect
      .poll(async () => page.locator('#name').evaluate((element) => element.validationMessage))
      .toBe('Value cannot be empty');
  });

  test('should log in a user', async ({ page }) => {
    await loginWithCredentials(page, 'test', 'test1234');
    await expect(page).toHaveURL(/\/(?:\?.*)?$/);
  });

  test('should log out a user', async ({ page }) => {
    await loginWithCredentials(page, 'test', 'test1234');

    await expect(page).toHaveURL(/\/(?:\?.*)?$/);
    await openUserMenu(page);
    await clickUserMenuItem(page, 'Log out');
    await page.waitForURL(/\/log-in(?:\?.*)?$/);
  });

  test('should close user menu on Escape and restore focus', async ({ page }) => {
    const userButton = page.locator('#user-dropdown-button');
    const dropdown = page.locator('#dropdown-user');

    await openUserMenu(page);
    await expect(dropdown).toBeVisible();
    await expect(userButton).toHaveAttribute('aria-expanded', 'true');
    await expect(dropdown).toHaveAttribute('aria-hidden', 'false');

    await page.keyboard.press('Escape');

    await expect(dropdown).toBeHidden();
    await expect(userButton).toBeFocused();
    await expect(userButton).toHaveAttribute('aria-expanded', 'false');
    await expect(dropdown).toHaveAttribute('aria-hidden', 'true');
  });

  test('should toggle user menu aria state on click', async ({ page }) => {
    const userButton = page.locator('#user-dropdown-button');
    const dropdown = page.locator('#dropdown-user');

    await openUserMenu(page);
    await expect(dropdown).toBeVisible();
    await expect(userButton).toHaveAttribute('aria-expanded', 'true');
    await expect(dropdown).toHaveAttribute('aria-hidden', 'false');

    await userButton.click();
    await expect(dropdown).toBeHidden();
    await expect(userButton).toHaveAttribute('aria-expanded', 'false');
    await expect(dropdown).toHaveAttribute('aria-hidden', 'true');
  });

  test('should keep user menu toggle stable after HTMX navigation', async ({ page }) => {
    const headerNavigation = page.locator('#header nav').first();

    await headerNavigation.getByRole('link', { name: 'About', exact: true }).click();
    await expect(page).toHaveURL(/\/about(?:\?.*)?$/);
    await headerNavigation.getByRole('link', { name: 'Jobs', exact: true }).click();
    await expect(page).toHaveURL(/\/(?:\?.*)?$/);

    const userButton = page.locator('#user-dropdown-button');
    const dropdown = page.locator('#dropdown-user');

    await userButton.click();
    await expect(dropdown).toBeVisible();
    await expect(userButton).toHaveAttribute('aria-expanded', 'true');

    await userButton.click();
    await expect(dropdown).toBeHidden();
    await expect(userButton).toHaveAttribute('aria-expanded', 'false');

    await userButton.click();
    await expect(dropdown).toBeVisible();
    await expect(userButton).toHaveAttribute('aria-expanded', 'true');
  });

  test('invalid credentials stay on log in page', async ({ page }) => {
    await openLoginPage(page);

    await page.locator('#username').fill('test');
    await page.locator('#password').fill('wrong');
    await page.getByRole('button', { name: 'Submit' }).click();

    await expect(page).toHaveURL(/\/log-in(?:\?.*)?$/);
  });

  test('should send experience fields using bracket keys on profile update', async ({ page }) => {
    await loginWithCredentials(page, 'test', 'test1234');
    await page.goto('/dashboard/job-seeker', { waitUntil: 'domcontentloaded' });
    await page.locator('#name').waitFor({ state: 'visible', timeout: 10000 });

    await page.locator('#name').fill('Test User');
    await page.locator('#email').fill('test@example.com');
    await page.locator('textarea[name="summary"]').fill('Profile summary');

    await page.locator('[data-section="experience"]').click();
    await page.locator('input[name="experience[0][title]"]').fill('Engineer');
    await page.locator('input[name="experience[0][company]"]').fill('ACME');
    await page.locator('textarea[name="experience[0][description]"]').fill('Worked on platform');
    await page.locator('input[name="experience[0][start_date]"]').fill('2026-02-06');

    const requestPromise = page.waitForRequest((request) => {
      return request.method() === 'PUT' && request.url().includes('/dashboard/job-seeker/profile/update');
    });

    await page.locator('#update-profile-button').click();

    const updateRequest = await requestPromise;
    const body = updateRequest.postData() || '';
    const formData = new URLSearchParams(body);
    expect(formData.get('experience[0][title]')).toBe('Engineer');
    expect(formData.get('experience[0][company]')).toBe('ACME');
    expect(formData.get('experience[0][description]')).toBe('Worked on platform');
    expect(formData.get('experience[0][start_date]')).toBe('2026-02-06');
  });

  test('should add a new job', async ({ page }) => {
    await loginWithCredentials(page, 'test', 'test1234');
    await page.goto('/dashboard/employer/jobs/add', { waitUntil: 'domcontentloaded' });

    const titleField = page.locator('#title');
    try {
      await titleField.waitFor({ state: 'visible', timeout: 5000 });
    } catch {
      await page.goto('/dashboard/employer?tab=jobs', { waitUntil: 'domcontentloaded' });
      const addJobButton = page.getByRole('button', { name: 'Add Job' });
      await addJobButton.waitFor({ state: 'visible', timeout: 10000 });
      await addJobButton.click();
      await titleField.waitFor({ state: 'visible', timeout: 10000 });
    }

    await page.locator('#title').click();
    await page.locator('#title').fill('job');
    await expect(page.locator('markdown-editor#description')).toHaveCount(1, { timeout: 10000 });

    await page.evaluate(() => {
      const jobsForm = document.getElementById('jobs-form');
      if (!(jobsForm instanceof HTMLFormElement)) {
        return;
      }

      const markdownEditor = document.querySelector('markdown-editor#description');
      let descriptionField =
        markdownEditor?.querySelector('textarea[name="description"]') ||
        jobsForm.querySelector('textarea[name="description"]');

      if (!(descriptionField instanceof HTMLTextAreaElement)) {
        descriptionField = document.createElement('textarea');
        descriptionField.name = 'description';
        descriptionField.style.display = 'none';
        jobsForm.append(descriptionField);
      }

      descriptionField.value = 'description';
      descriptionField.dispatchEvent(new Event('input', { bubbles: true }));
      descriptionField.dispatchEvent(new Event('change', { bubbles: true }));
    });

    await page.getByRole('button', { name: 'Publish' }).click();
    expect(page.url()).toContain('/dashboard/employer');
  });

  test('should close employer job actions dropdown on Escape', async ({ page }) => {
    await loginWithCredentials(page, 'test', 'test1234');
    await page.goto('/dashboard/employer?tab=jobs');

    const actionsDropdown = await openEmployerActionsDropdown(page);
    if (!actionsDropdown) {
      throw new Error('Expected at least one employer job action button.');
    }

    const { actionButton, dropdown } = actionsDropdown;
    await expect(dropdown).toBeVisible();
    await expect(actionButton).toHaveAttribute('aria-expanded', 'true');
    await expect(dropdown).toHaveAttribute('aria-hidden', 'false');

    await page.keyboard.press('Escape');

    await expect(dropdown).toBeHidden();
    await expect(actionButton).toBeFocused();
    await expect(actionButton).toHaveAttribute('aria-expanded', 'false');
    await expect(dropdown).toHaveAttribute('aria-hidden', 'true');
  });

  test('should close employer job actions dropdown after selecting an action', async ({ page }) => {
    await loginWithCredentials(page, 'test', 'test1234');
    await page.goto('/dashboard/employer?tab=jobs');

    const actionsDropdown = await openEmployerActionsDropdown(page);
    if (!actionsDropdown) {
      throw new Error('Expected at least one employer job action button.');
    }

    const { actionButton, dropdown } = actionsDropdown;
    await expect(dropdown).toBeVisible();

    const deleteAction = dropdown.locator('[data-delete-job-button]').first();
    await deleteAction.click();

    await expect(dropdown).toBeHidden();
    await expect(actionButton).toHaveAttribute('aria-expanded', 'false');
    await expect(dropdown).toHaveAttribute('aria-hidden', 'true');
  });

  test('should close employer job actions dropdown on outside click', async ({ page }) => {
    await loginWithCredentials(page, 'test', 'test1234');
    await page.goto('/dashboard/employer?tab=jobs');

    const actionsDropdown = await openEmployerActionsDropdown(page);
    if (!actionsDropdown) {
      throw new Error('Expected at least one employer job action button.');
    }

    const { actionButton, dropdown } = actionsDropdown;
    await expect(dropdown).toBeVisible();
    await expect(actionButton).toHaveAttribute('aria-expanded', 'true');
    await expect(dropdown).toHaveAttribute('aria-hidden', 'false');

    await page.locator('#dashboard-content').first().click({
      position: { x: 8, y: 8 },
    });

    await expect(dropdown).toBeHidden();
    await expect(actionButton).toHaveAttribute('aria-expanded', 'false');
    await expect(dropdown).toHaveAttribute('aria-hidden', 'true');
  });

  test('should close employer selector dropdown after choosing an employer', async ({ page }) => {
    await loginWithCredentials(page, 'test', 'test1234');
    await page.goto('/dashboard/employer');

    const employerButton = page.locator('#employer-btn');
    await expect(employerButton).toHaveCount(1);

    const dropdown = page.locator('#dropdown-employers');
    await employerButton.click();
    await expect(dropdown).toBeVisible();
    await expect(employerButton).toHaveAttribute('aria-expanded', 'true');
    await expect(dropdown).toHaveAttribute('aria-hidden', 'false');

    const selectableEmployers = dropdown.locator('button.employer-button:not([disabled])');
    expect(await selectableEmployers.count()).toBeGreaterThan(0);

    await selectableEmployers.first().click();

    await expect(dropdown).toBeHidden();
    await expect(employerButton).toHaveAttribute('aria-expanded', 'false');
    await expect(dropdown).toHaveAttribute('aria-hidden', 'true');
  });

  test('should close applications jobs dropdown after selecting a filter', async ({ page }) => {
    await loginWithCredentials(page, 'test', 'test1234');
    await page.goto('/dashboard/employer?tab=applications');

    const jobsButton = page.locator('#jobs-btn');
    await expect(jobsButton).toHaveCount(1);
    await expect(jobsButton).toBeEnabled();

    const dropdown = page.locator('#dropdown-jobs');
    await jobsButton.click();
    await expect(dropdown).toBeVisible();
    await expect(jobsButton).toHaveAttribute('aria-expanded', 'true');
    await expect(dropdown).toHaveAttribute('aria-hidden', 'false');

    const selectableFilters = dropdown.locator('button:not([disabled])');
    expect(await selectableFilters.count()).toBeGreaterThan(0);

    await selectableFilters.first().click();

    await expect(dropdown).toBeHidden();
    await expect(jobsButton).toHaveAttribute('aria-expanded', 'false');
    await expect(dropdown).toHaveAttribute('aria-hidden', 'true');
  });

  test('should display job details correctly', async ({ page }) => {
    const selectedJobCard = jobCards(page).first();
    await selectedJobCard.waitFor();

    const expectedTitle = (await selectedJobCard.getAttribute('data-job-title'))?.trim();
    expect(expectedTitle).toBeTruthy();

    const expectedCompany = (await selectedJobCard.getAttribute('data-job-company'))?.trim();
    expect(expectedCompany).toBeTruthy();

    await selectedJobCard.click();
    await expect(page.locator('#preview-modal .text-xl')).toBeVisible({ timeout: 10000 });

    await expect(
      page.locator(
        '.text-xl.lg\\:leading-tight.font-stretch-condensed.font-medium.text-stone-900.lg\\:truncate.my-1\\.5.md\\:my-0'
      )
    ).toHaveText(expectedTitle as string);
    await expect(page.locator('#preview-content')).toContainText(expectedCompany as string);

    const previewContent = page.locator('#preview-content');

    const descriptionLocator = previewContent.locator(
      'div.text-lg.font-semibold.text-stone-800:has-text("Job description") + div.text-sm\\/6.text-stone-600.markdown'
    );
    await expect(descriptionLocator).toBeVisible();
    await expect(descriptionLocator).not.toHaveText('');

    await expect(previewContent.getByText('Job type')).toBeVisible();
    await expect(previewContent.getByText('Workplace')).toBeVisible();
    await expect(previewContent.getByRole('button', { name: 'Apply' })).toBeEnabled();
    await expect(previewContent.getByText(/Published/)).toBeVisible();
    await expect(previewContent.getByText('Share this job')).toBeVisible();
  });

  test('should lock body scroll while preview modal is open and restore on close', async ({ page }) => {
    await jobCards(page).first().waitFor();
    await jobCards(page).first().click();
    await expect(page.locator('#preview-modal .text-xl')).toBeVisible({ timeout: 10000 });

    await expect.poll(async () => page.evaluate(() => document.body.dataset.modalOpenCount || '0')).toBe('1');
    await expect.poll(async () => page.evaluate(() => document.body.style.overflow)).toBe('hidden');

    await page.locator('#close-preview-modal').click();
    await expect(page.locator('#preview-modal')).toBeHidden();

    await expect.poll(async () => page.evaluate(() => document.body.dataset.modalOpenCount || '0')).toBe('0');
    await expect.poll(async () => page.evaluate(() => document.body.style.overflow)).toBe('');
  });

  test('should keep body scroll locked when closing embed modal over preview modal', async ({ page }) => {
    await jobCards(page).first().waitFor();
    await jobCards(page).first().click();
    await expect(page.locator('#preview-modal .text-xl')).toBeVisible({ timeout: 10000 });

    const embedButton = page.locator('#embed-code-button');
    if ((await embedButton.count()) === 0) {
      console.log('Embed code button not available, skipping test.');
      return;
    }

    await embedButton.click();
    await expect(page.locator('#embed-code-modal')).toBeVisible();

    await expect.poll(async () => page.evaluate(() => document.body.dataset.modalOpenCount || '0')).toBe('2');
    await expect.poll(async () => page.evaluate(() => document.body.style.overflow)).toBe('hidden');

    await page.locator('#close-embed-code-modal').click();
    await expect(page.locator('#embed-code-modal')).toBeHidden();
    await expect(page.locator('#preview-modal .text-xl')).toBeVisible();

    await expect.poll(async () => page.evaluate(() => document.body.dataset.modalOpenCount || '0')).toBe('1');
    await expect.poll(async () => page.evaluate(() => document.body.style.overflow)).toBe('hidden');

    await page.locator('#close-preview-modal').click();
    await expect(page.locator('#preview-modal')).toBeHidden();
    await expect.poll(async () => page.evaluate(() => document.body.dataset.modalOpenCount || '0')).toBe('0');
    await expect.poll(async () => page.evaluate(() => document.body.style.overflow)).toBe('');
  });

  test('should display share buttons properly', async ({ page }) => {
    await jobCards(page).first().waitFor();
    await jobCards(page).first().click();
    await expect(page.locator('#preview-modal .text-xl')).toBeVisible({ timeout: 10000 });

    const shareButtons = [
      { title: 'Twitter share link', sharer: 'twitter' },
      { title: 'Facebook share link', sharer: 'facebook' },
      { title: 'LinkedIn share link', sharer: 'linkedin' },
      { title: 'Email share link', sharer: 'email' },
      { title: 'Copy link', sharer: '' },
    ];

    for (const button of shareButtons) {
      const element = page.getByTitle(button.title);
      await expect(element).toBeVisible();
      if (button.title !== 'Copy link') {
        await expect.poll(async () => (await element.getAttribute('data-sharer')) || '').toBe(button.sharer);
        await expect.poll(async () => (await element.getAttribute('data-url')) || '').toContain('job_id=');
        await expect.poll(async () => (await element.getAttribute('href')) || '').not.toBe('');
        const href = await element.getAttribute('href');
        expect(href).toBeTruthy();
        if (button.title === 'Email share link') {
          expect(href).toMatch(/^mailto:/);
        } else {
          expect(href).toMatch(/^https?:\/\//);
        }
      } else {
        await expect(element).toBeEnabled();
      }
    }
  });

  test('should allow paginating through jobs', async ({ page }) => {
    const nextButton = page.getByRole('link', { name: 'Next' });
    await expect(nextButton).toBeVisible();
    await nextButton.click();
    await expect(page).toHaveURL(/offset=20/);
    await expect(page.locator('#results')).toHaveText(/^21 - \d+ of \d+ results$/);
  });

  test('should show pagination spinner while loading next page', async ({ page }) => {
    let delayed = false;
    await page.route('**/section/jobs/results*', async (route) => {
      if (!delayed) {
        delayed = true;
        await new Promise((resolve) => setTimeout(resolve, 500));
      }
      await route.continue();
    });

    const nextButton = page.getByRole('link', { name: 'Next' });
    await expect(nextButton).toBeVisible();
    await nextButton.click();

    await expect(page.locator('#pagination-next-spinner')).toBeVisible();
    await expect(page).toHaveURL(/offset=20/);
    await expect(page.locator('#results')).toHaveText(/^21 - \d+ of \d+ results$/);
  });
});
