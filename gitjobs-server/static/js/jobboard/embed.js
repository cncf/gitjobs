import { trackSearchAppearances } from "/static/js/common/common.js";

/**
 * Tracks search appearances for jobs rendered on the embed page.
 */
export const initializeEmbedJobsPage = () => {
  const jobCards = document.querySelectorAll("[data-embed-job-id]");
  const jobIds = Array.from(jobCards)
    .map((jobCard) => jobCard.dataset.embedJobId)
    .filter(Boolean);

  if (jobIds.length > 0) {
    trackSearchAppearances(jobIds);
  }
};

initializeEmbedJobsPage();
