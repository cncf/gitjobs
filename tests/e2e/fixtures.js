import { expect, test as base } from "@playwright/test";

import { selectEmployerContext } from "./dashboard/employer/common/helpers.js";
import { loginWithCredentials } from "./shared/auth.js";
import { buildE2eUrl } from "./shared/navigation.js";

const EMPLOYER_ID = "18fff2d7-c794-4130-85e4-76b9d7c60b72";
const storageStateCache = new Map();

export const TEST_USERS = {
  applicant: {
    password: "test1234",
    userId: "11111111-1111-4111-8111-111111111104",
    username: "applicant",
  },
  employer: {
    password: "test1234",
    userId: "11111111-1111-4111-8111-111111111101",
    username: "employer",
  },
  jobSeeker: {
    password: "test1234",
    userId: "11111111-1111-4111-8111-111111111102",
    username: "jobseeker",
  },
  moderator: {
    password: "test1234",
    userId: "11111111-1111-4111-8111-111111111103",
    username: "moderator",
  },
};

/**
 * Applies the cached cookies and origin storage to a new test page.
 * @param {import("@playwright/test").Page} page - Page receiving the state.
 * @param {object} storageState - Cached Playwright browser storage state.
 * @returns {Promise<void>}
 */
const applyStorageState = async (page, storageState) => {
  await page.context().addCookies(storageState.cookies);

  await page.addInitScript((origins) => {
    const currentOrigin = origins.find(({ origin }) => origin === window.location.origin);
    for (const { name, value } of currentOrigin?.localStorage || []) {
      window.localStorage.setItem(name, value);
    }
  }, storageState.origins);
};

/**
 * Returns true when the page still has access to the seeded role dashboard.
 * @param {import("@playwright/test").Page} page - Authenticated test page.
 * @param {string} dashboardPath - Dashboard used to validate the session.
 * @returns {Promise<boolean>}
 */
const hasValidSession = async (page, dashboardPath) => {
  const response = await page.request.get(buildE2eUrl(dashboardPath), {
    maxRedirects: 0,
  });

  return response.status() === 200;
};

/**
 * Authenticates a seeded user when its cached browser state is unavailable.
 * @param {import("@playwright/test").Page} page - Page to authenticate.
 * @param {object} options - Seeded user and dashboard options.
 * @returns {Promise<void>}
 */
const prepareAuthenticatedPage = async (page, { dashboardPath, user }) => {
  const cachedStorageState = storageStateCache.get(user.username);
  if (cachedStorageState) {
    await applyStorageState(page, cachedStorageState);
    if (await hasValidSession(page, dashboardPath)) {
      return;
    }

    storageStateCache.delete(user.username);
    await page.context().clearCookies();
  }

  await loginWithCredentials(page, user.username, user.password);
  storageStateCache.set(user.username, await page.context().storageState());
};

/**
 * Builds a reusable authenticated page fixture for one seeded role.
 * @param {object} options - Seeded user and dashboard options.
 * @returns {Function}
 */
const authenticatedPageFixture =
  (options) =>
  async ({ page }, use) => {
    await prepareAuthenticatedPage(page, options);
    if (options.preparePage) {
      await options.preparePage(page);
    }
    await use(page);
  };

export const test = base.extend({
  applicantPage: authenticatedPageFixture({
    dashboardPath: "/dashboard/job-seeker",
    user: TEST_USERS.applicant,
  }),
  employerPage: authenticatedPageFixture({
    dashboardPath: "/dashboard/employer",
    preparePage: (page) => selectEmployerContext(page, EMPLOYER_ID),
    user: TEST_USERS.employer,
  }),
  jobSeekerPage: authenticatedPageFixture({
    dashboardPath: "/dashboard/job-seeker",
    user: TEST_USERS.jobSeeker,
  }),
  moderatorPage: authenticatedPageFixture({
    dashboardPath: "/dashboard/moderator",
    user: TEST_USERS.moderator,
  }),
});

export { expect };
