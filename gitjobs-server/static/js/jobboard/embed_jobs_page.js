import { trackSearchAppearances } from "/static/js/common/common.js";

/**
 * Initializes the embedded jobs page tracking.
 * @param {string[]} jobIds - IDs for jobs currently visible in embed results
 */
export const initializeEmbedJobsPage = (jobIds = []) => {
  const validJobIds = jobIds.filter(Boolean);
  if (validJobIds.length === 0) {
    return;
  }

  trackSearchAppearances(validJobIds);
};
