import { handleHtmxResponse } from "/static/js/common/alerts.js";
import {
  addParamToQueryString,
  bindHtmxAfterRequestOnce,
  trackerJobView,
  trackSearchAppearances,
} from "/static/js/common/common.js";
import { resetForm, updateResults } from "/static/js/jobboard/filters.js";

/**
 * Initializes no-results reset links and result-card interactions.
 * @param {Object} options - Initialization options
 * @param {boolean} options.hasJobs - Whether current result set has jobs
 * @param {string} options.currentPageContent - Results summary content
 * @param {string} [options.unavailableJobMessage] - Preview unavailable message
 */
export const initializeJobboardResults = ({
  hasJobs,
  currentPageContent,
  unavailableJobMessage = "This job is no longer available. It may have been removed recently.",
}) => {
  const resetDesktopFilters = document.getElementById("reset-link-desktop-filters");
  if (resetDesktopFilters && resetDesktopFilters.dataset.resetBound !== "true") {
    resetDesktopFilters.addEventListener("click", () => resetForm("desktop-jobs-form"));
    resetDesktopFilters.dataset.resetBound = "true";
  }

  const resetMobileFilters = document.getElementById("reset-link-mobile-filters");
  if (resetMobileFilters && resetMobileFilters.dataset.resetBound !== "true") {
    resetMobileFilters.addEventListener("click", () => resetForm("mobile-jobs-form"));
    resetMobileFilters.dataset.resetBound = "true";
  }

  bindHtmxAfterRequestOnce({
    selector: "[data-preview-job]",
    handler: (event) => {
      if (
        handleHtmxResponse({
          xhr: event.detail.xhr,
          errorMessage: unavailableJobMessage,
        })
      ) {
        const previewButton = event.currentTarget;
        if (!(previewButton instanceof HTMLElement)) {
          return;
        }

        const jobId = previewButton.dataset.jobId;
        if (!jobId) {
          return;
        }

        addParamToQueryString("job_id", jobId, { modal_preview: true });

        // Register views only on user-triggered open, not history popstate.
        const triggerType = event.detail?.requestConfig?.triggeringEvent?.type || "";
        if (["open-modal", "click"].includes(triggerType)) {
          trackerJobView(jobId);
        }
      }
    },
    boundAttribute: "previewBound",
  });

  updateResults(currentPageContent);

  if (hasJobs) {
    const jobButtons = document.querySelectorAll("[data-job-id]");
    const jobIds = Array.from(jobButtons)
      .map((button) => button.dataset.jobId)
      .filter(Boolean);
    if (jobIds.length > 0) {
      trackSearchAppearances(jobIds);
    }
  }
};
