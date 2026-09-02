const BASE_URL = process.env.BASE_URL || "http://localhost:9000";

/**
 * Builds an absolute URL for requests made outside a page navigation.
 * @param {string} path - Application path.
 * @returns {string}
 */
export const buildE2eUrl = (path) => {
  return new URL(path, BASE_URL).toString();
};

/**
 * Navigates home while tolerating transient server readiness failures.
 * @param {import("@playwright/test").Page} page - Page to navigate.
 * @returns {Promise<void>}
 */
export const navigateHomeWithRetries = async (page) => {
  await navigateWithRetries(page, "/", "home page");
};

/**
 * Navigates to a path while tolerating transient server readiness failures.
 * @param {import("@playwright/test").Page} page - Page to navigate.
 * @param {string} path - Application path.
 * @param {string} [label=path] - Human-readable destination.
 * @returns {Promise<void>}
 */
export const navigateWithRetries = async (page, path, label = path) => {
  let lastError;
  for (let attempt = 0; attempt < 3; attempt++) {
    try {
      await page.goto(buildE2eUrl(path), {
        timeout: 9000,
        waitUntil: "domcontentloaded",
      });
      return;
    } catch (error) {
      lastError = error;
    }
  }

  throw new Error(`Failed to navigate to ${label} after 3 attempts: ${String(lastError)}`);
};

/**
 * Runs an action and verifies the matching HTTP response.
 * Status is checked after matching so endpoint failures surface immediately.
 * @param {import("@playwright/test").Page} page - Page performing the action.
 * @param {() => Promise<unknown>} action - Action that triggers the response.
 * @param {object} options - Response matching options.
 * @param {string} options.method - Expected HTTP method.
 * @param {number} [options.status] - Expected status or any successful status.
 * @param {string} [options.urlEndsWith] - Expected URL suffix.
 * @param {string} [options.urlIncludes] - Expected URL fragment.
 * @returns {Promise<import("@playwright/test").Response>}
 */
export const waitForActionResponse = async (page, action, { method, status, urlEndsWith, urlIncludes }) => {
  const [response] = await Promise.all([
    page.waitForResponse((candidate) => {
      return (
        candidate.request().method() === method &&
        (!urlIncludes || candidate.url().includes(urlIncludes)) &&
        (!urlEndsWith || candidate.url().endsWith(urlEndsWith))
      );
    }),
    action(),
  ]);

  const hasExpectedStatus = status === undefined ? response.ok() : response.status() === status;
  if (!hasExpectedStatus) {
    throw new Error(
      `Expected ${method} ${response.url()} to ${
        status === undefined ? "succeed" : `return ${status}`
      }, received ${response.status()}.`,
    );
  }

  return response;
};
