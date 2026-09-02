import { expect } from "@playwright/test";

export const JOB_CARD_SELECTOR = '[data-preview-job="true"]';
export const JOB_TITLE_SELECTOR = '[data-testid="job-card-title"]';

/**
 * Returns the visible job result cards.
 * @param {import("@playwright/test").Page} page - Job board page.
 * @returns {import("@playwright/test").Locator}
 */
export const jobCards = (page) => {
  return page.locator(JOB_CARD_SELECTOR);
};

/**
 * Returns the job title elements within the result list.
 * @param {import("@playwright/test").Page} page - Job board page.
 * @returns {import("@playwright/test").Locator}
 */
export const jobTitles = (page) => {
  return page.locator(JOB_TITLE_SELECTOR);
};

/**
 * Returns the job type controls rendered in result cards.
 * @param {import("@playwright/test").Page} page - Job board page.
 * @returns {import("@playwright/test").Locator}
 */
export const jobTypeButtons = (page) => {
  return page.getByRole("button", { name: /Job type/ });
};

/**
 * Returns the public job search field.
 * @param {import("@playwright/test").Page} page - Job board page.
 * @returns {import("@playwright/test").Locator}
 */
export const searchInput = (page) => {
  return page.locator('input[placeholder="Search jobs"]');
};

/**
 * Waits until every result has the expected job type.
 * @param {import("@playwright/test").Page} page - Job board page.
 * @param {string} expectedType - Expected result type.
 * @returns {Promise<void>}
 */
export const waitForOnlyJobTypeResults = async (page, expectedType) => {
  const jobTypePattern = buildJobTypePattern(expectedType);

  await expect
    .poll(
      async () => {
        const cards = jobCards(page);
        const totalCards = await cards.count();
        if (totalCards === 0) {
          return false;
        }

        const matchingCards = await page.locator(JOB_CARD_SELECTOR, { hasText: jobTypePattern }).count();
        return totalCards === matchingCards;
      },
      { timeout: 10000 },
    )
    .toBe(true);
};

/**
 * Waits until every result has one of the expected job types.
 * @param {import("@playwright/test").Page} page - Job board page.
 * @param {string[]} expectedTypes - Expected result types.
 * @returns {Promise<void>}
 */
export const waitForOnlyJobTypeSetResults = async (page, expectedTypes) => {
  const typePatterns = expectedTypes.map(buildJobTypePattern);

  await expect
    .poll(
      async () => {
        const cards = jobCards(page);
        const totalCards = await cards.count();
        if (totalCards === 0) {
          return false;
        }

        const matchesByType = await Promise.all(
          typePatterns.map((pattern) => page.locator(JOB_CARD_SELECTOR, { hasText: pattern }).count()),
        );
        const totalMatches = matchesByType.reduce((sum, value) => sum + value, 0);
        return totalMatches === totalCards;
      },
      { timeout: 10000 },
    )
    .toBe(true);
};

/**
 * Builds a whitespace-tolerant pattern for a visible job type.
 * @param {string} jobType - Visible job type.
 * @returns {RegExp}
 */
const buildJobTypePattern = (jobType) => {
  const normalizedJobType = jobType
    .trim()
    .split(/\s+/)
    .map((word) => word.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"))
    .join("\\s+");

  return new RegExp(`Job\\s*type\\s*${normalizedJobType}`, "i");
};
