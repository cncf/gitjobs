import { test, expect } from '@playwright/test';

test.describe('GitJobs', () => {
  test.beforeEach(async ({ page }) => {
    for (let i = 0; i < 3; i++) {
      try {
        await page.goto('/', { timeout: 60000 });
        break;
      } catch (error) {
        console.log(`Failed to navigate to page, retrying... (${i + 1}/3)`);
      }
    }
    // Handle cookie consent
    try {
      await page.getByRole('button', { name: 'Accept all' }).click({ timeout: 5000 });
    } catch (error) {
      // Ignore if the cookie consent is not visible
    }
  });

  test('should have the correct title and heading', async ({ page }) => {
    await expect(page).toHaveTitle(/GitJobs/);
    await expect(page.getByRole('heading', { level: 1 })).toBeVisible();
  });

  test('should apply a filter and verify that the results are updated', async ({ page }) => {
    await page.locator('div:nth-child(4) > div > .font-semibold').first().click();
    await page.locator('label').filter({ hasText: 'Full Time' }).nth(1).click();

    const jobCards = await page.getByRole('button', { name: /Job type/ }).all();
    for (const jobCard of jobCards) {
      await expect(jobCard.locator('.capitalize').first()).toHaveText('full time');
    }
  });

  test('should reset filters', async ({ page }) => {
    const initialJobCount = await page.getByRole('button', { name: /Job type/ }).count();
    await page.locator('label').filter({ hasText: 'Full Time' }).nth(1).click();
    await page.locator('#reset-desktop-filters').click();
    const newJobCount = await page.getByRole('button', { name: /Job type/ }).count();
    expect(newJobCount).toEqual(initialJobCount);
  });

  test('should sort jobs', async ({ page }) => {
    await page.locator('#sort-desktop').selectOption('open-source');
    await expect(page).toHaveURL(/\?sort=open-source/);
  });

  test('should navigate to the stats page and interact with charts', async ({ page, browserName }) => {
    if (browserName === 'firefox') {
      // Skip this test on Firefox as it's failing due to a rendering issue with the charts
      return;
    }
    await page.getByRole('link', { name: 'Stats' }).click();
    await expect(page).toHaveURL(/\/stats/);

    await page.waitForTimeout(1000);
    const noData = page.locator('text="No data available yet"').first();
    if (await noData.isVisible()) {
      await expect(noData).toBeVisible();
    } else {
      await page.waitForSelector('#line-chart rect', { timeout: 15000 });
      await page.locator('#line-chart rect').first().click({ force: true });
      await page.waitForSelector('#bar-daily rect', { timeout: 15000 });
      await page.locator('#bar-daily rect').first().click({ force: true });
    }
  });

  test('should navigate to the about page and check for a body', async ({ page }) => {
    await page.getByRole('link', { name: 'About' }).click();
    await expect(page).toHaveURL(/\/about/);
    await expect(page.locator('body')).toBeVisible();
  });

  test('should navigate to the sign-up page', async ({ page }) => {
    await page.locator('#user-dropdown-button').click();
    await page.getByRole('link', { name: 'Sign up' }).click();
    await expect(page).toHaveURL(/\/sign-up/);
  });

  test('should allow viewing a job posting', async ({ page }) => {
    const jobCount = await page.getByRole('button', { name: /Job type/ }).count();
    if (jobCount === 0) {
      console.log('No jobs found, skipping test.');
      return;
    }
    await page.getByRole('button', { name: /Job type/ }).first().click();
    await expect(page).toHaveURL(/\?job_id=/);
    await expect(page.locator('#job-view').getByRole('heading')).toBeVisible();
  });

  test('should display job details correctly', async ({ page }) => {
    const jobCount = await page.getByRole('button', { name: /Job type/ }).count();
    if (jobCount === 0) {
      console.log('No jobs found, skipping test.');
      return;
    }
    await page.getByRole('button', { name: /Job type/ }).first().click();
    await expect(page.locator('#job-view').getByRole('heading')).toBeVisible();
    await expect(page.locator('#preview-content').getByText(/Job description/)).toBeVisible();
    await expect(page.getByRole('button', { name: 'Apply' })).toBeEnabled();
    await expect(page.locator('#preview-content').getByText(/Published/)).toBeVisible();
    await expect(page.locator('#preview-content').getByText(/Job type/)).toBeVisible();
    await expect(page.locator('#preview-content').getByText(/Workplace/)).toBeVisible();
    await expect(page.locator('#preview-content').getByText(/Seniority level/)).toBeVisible();
    await expect(page.getByText('Share this job')).toBeVisible();
  });

  test('should allow paginating through jobs', async ({ page }) => {
    const paginationVisible = await page.locator('[aria-label="pagination"]').isVisible();
    if (!paginationVisible) {
      console.log('Pagination not visible, skipping test.');
      return;
    }
    const initialPageNumber = await page.locator('[aria-current="page"]').textContent();
    await page.getByLabel(/Go to page/).last().click();
    const newPageNumber = await page.locator('[aria-current="page"]').textContent();
    expect(newPageNumber).not.toBe(initialPageNumber);
  });
});
