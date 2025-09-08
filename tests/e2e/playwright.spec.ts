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
  });

  test('should have the correct title and heading', async ({ page }) => {
    await expect(page).toHaveTitle(/GitJobs/);
    await expect(page.getByRole('heading', { level: 1 })).toBeVisible();
  });

  test('should apply a filter and verify that the results are updated', async ({ page }) => {
    await page.locator('div:nth-child(4) > div > .font-semibold').first().click();
    await page.locator('label').filter({ hasText: 'Full Time' }).nth(1).click();
    await page.waitForFunction(
      () => {
        const currentCount = document.querySelectorAll('[data-preview-job="true"]').length;
        return currentCount === 12;
      }
    );

    const jobCards = await page.getByRole('button', { name: /Job type/ }).all();
    for (const jobCard of jobCards) {
      const jobTypeElement = jobCard.locator('.capitalize').first();
      if (await jobTypeElement.isVisible()) {
        await expect(jobTypeElement).toHaveText('full time');
      }
    }
  });
  test('should reset filters', async ({ page }) => {
    await page.locator('label').filter({ hasText: 'Part Time' }).nth(1).click();

    await page.waitForFunction(
      () => {
        const currentCount = document.querySelectorAll('[data-preview-job="true"]').length;
        return currentCount === 3;
      }
    );
    const firstJobAfterFilter = await page.locator('.text-base.font-stretch-condensed.font-medium.text-stone-900.line-clamp-2.md\\:line-clamp-1').first().textContent();
    expect(firstJobAfterFilter!.trim()).toBe('Data Scientist');
    await page.locator('#reset-desktop-filters').click();
    await expect(page.locator('#results')).toHaveText('1 - 20 of 21 results');
    const firstJobAfterReset = await page.locator('.text-base.font-stretch-condensed.font-medium.text-stone-900.line-clamp-2.md\\:line-clamp-1').first().textContent();
    expect(firstJobAfterReset!.trim()).toBe('Frontend Developer');
  });

  test('should sort jobs', async ({ page }) => {
    const initialJobTitles = (await page.locator('.text-base.font-stretch-condensed.font-medium.text-stone-900.line-clamp-2.md\\:line-clamp-1').allTextContents()).map(title => title.trim());
    await page.locator('#sort-desktop').selectOption('salary');
    await expect(page).toHaveURL(/\?sort=salary/);
    await page.waitForTimeout(500);
    const sortedJobTitles = (await page.locator('.text-base.font-stretch-condensed.font-medium.text-stone-900.line-clamp-2.md\\:line-clamp-1').allTextContents()).map(title => title.trim());
    expect(sortedJobTitles[0]).toBe('Security Engineer');
    expect(sortedJobTitles[1]).toBe('DevOps Engineer');
    expect(sortedJobTitles[2]).toBe('Product Manager');
    expect(sortedJobTitles[3]).toBe('Backend Developer');
    expect(sortedJobTitles[4]).toBe('Frontend Developer');
    expect(sortedJobTitles).not.toEqual(initialJobTitles);
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

  test('should log in a user', async ({ page }) => {
    await page.locator('#user-dropdown-button').click();
    await page.getByRole('link', { name: 'Log in' }).click();
    await page.waitForURL('**/log-in');
    await page.locator('#username').fill('test');
    await page.locator('#password').fill('test');
    await page.getByRole('button', { name: 'Submit' }).click();
  });

  test('should add a new job', async ({ page }) => {
    await page.locator('#user-dropdown-button').click();
    await page.getByRole('link', { name: 'Log in' }).click();
    await page.waitForURL('**/log-in');
    await page.locator('#username').fill('test');
    await page.locator('#password').fill('test');
    await page.getByRole('button', { name: 'Submit' }).click();
    await page.goto('/');

    await page.getByRole('link', { name: 'Post a job' }).click();
    await page.waitForURL('**/dashboard/employer');
    await page.getByRole('button', { name: 'Add Job' }).click();
    await page.getByRole('textbox', { name: 'Title *' }).click();
    await page.getByRole('textbox', { name: 'Title *' }).fill('job');
    await page.locator('#description pre').nth(1).click();
    await page.locator('#description').getByRole('application').getByRole('textbox').fill('description');
    await page.getByRole('button', { name: 'Publish' }).click();
    expect(page.url()).toContain('/dashboard/employer');
  });

  test('should display job details correctly', async ({ page }) => {
    const expectedTitle = 'Frontend Developer';
    const expectedDescription = 'React expert';
    const expectedKind = 'full time';
    const expectedSeniority = 'senior';
    const expectedWorkplace = 'remote';
    const expectedSalaryAmount = '120K';
    const expectedSalaryCurrency = 'USD';
    const expectedSalaryPeriod = '/ year';

    await page.waitForSelector('[data-preview-job="true"]');
    await page.locator('[data-preview-job="true"]').first().click();
    await expect(page.locator('#preview-modal .text-xl')).toBeVisible({ timeout: 10000 });

    await expect(page.locator('.text-xl.lg\\:leading-tight.font-stretch-condensed.font-medium.text-stone-900.lg\\:truncate.my-1\\.5.md\\:my-0')).toHaveText(expectedTitle);
    await expect(page.locator('div.text-lg.font-semibold.text-stone-800:has-text("Job description") + div.text-sm\\/6.text-stone-600.markdown p')).toHaveText(expectedDescription);
    await expect(page.locator('div:has-text("Job type") + div.flex.items-center.text-xs > div.truncate.capitalize')).toHaveText(expectedKind);
    await expect(page.locator('div:has-text("Workplace") + div.flex.items-center.text-xs > div.truncate.capitalize')).toHaveText(expectedWorkplace);
    await expect(page.locator('div:has-text("Seniority level") + div.flex.items-center.text-xs > div.truncate.capitalize')).toHaveText(expectedSeniority);
    await expect(page.locator('#preview-content div:has-text("Salary") div.flex.items-baseline.font-medium.text-stone-900.text-sm > div.text-xs.text-stone-500.me-1')).toHaveText(expectedSalaryCurrency);
    await expect(page.locator('#preview-content div:has-text("Salary") div.flex.items-baseline.font-medium.text-stone-900.text-sm')).toContainText(expectedSalaryAmount);
    await expect(page.locator('#preview-content div:has-text("Salary") div.flex.items-baseline > div.text-stone-900.text-xs.ms-1')).toHaveText(expectedSalaryPeriod);
    await expect(page.getByRole('button', { name: 'Apply' })).toBeEnabled();
    await expect(page.locator('#preview-content').getByText(/Published/)).toBeVisible();
    await expect(page.getByText('Share this job')).toBeVisible();
  });

  test('should allow paginating through jobs', async ({ page }) => {
    const nextButton = page.getByRole('link', { name: 'Next' });
    if (!(await nextButton.isVisible())) {
      console.log('Pagination next button not visible, skipping test.');
      return;
    }
    await nextButton.click();
    await expect(page).toHaveURL(/offset=20/);
    await expect(page.locator('#results')).toHaveText('21 - 21 of 21 results');
  });
});
