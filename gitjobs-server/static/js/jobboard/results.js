import { handleHtmxResponse } from "/static/js/common/alerts.js";
import { addParamToQueryString, trackerJobView, trackSearchAppearances } from "/static/js/common/common.js";
import { resetForm, updateResults } from "/static/js/jobboard/filters.js";

/**
 * Initializes no-results reset links and result-card interactions.
 * @param {Object} options - Initialization options
 * @param {boolean} options.hasJobs - Whether current result set has jobs
 * @param {string} options.currentPageContent - Current results summary content
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

  const previewButtons = document.querySelectorAll("[data-preview-job]");
  previewButtons.forEach((button) => {
    if (button.dataset.previewBound === "true") {
      return;
    }

    button.addEventListener("htmx:afterRequest", (event) => {
      if (
        handleHtmxResponse({
          xhr: event.detail.xhr,
          errorMessage: unavailableJobMessage,
        })
      ) {
        const jobId = button.dataset.jobId;
        addParamToQueryString("job_id", jobId, { modal_preview: true });

        // Register views only on user-triggered open, not history popstate.
        const triggerType = event.detail?.requestConfig?.triggeringEvent?.type || "";
        if (["open-modal", "click"].includes(triggerType)) {
          trackerJobView(jobId);
        }
      }
    });

    button.dataset.previewBound = "true";
  });

  if (hasJobs) {
    updateResults(currentPageContent);

    const jobButtons = document.querySelectorAll("[data-job-id]");
    const jobIds = Array.from(jobButtons)
      .map((button) => button.dataset.jobId)
      .filter(Boolean);
    if (jobIds.length > 0) {
      trackSearchAppearances(jobIds);
    }
    return;
  }

  updateResults("");
};

/**
 * Initializes results behavior from server-rendered config elements.
 * @param {Document|HTMLElement} [root=document] - Root where config is searched
 */
export const initializeJobboardResultsFromDom = (root = document) => {
  const selector = '[data-jobboard-results-config="true"]';
  const resultConfigs = [];

  if (root instanceof Element && root.matches(selector)) {
    resultConfigs.push(root);
  }

  if (typeof root.querySelectorAll === "function") {
    resultConfigs.push(...root.querySelectorAll(selector));
  }

  resultConfigs.forEach((resultConfig) => {
    if (resultConfig.dataset.jobboardResultsBound === "true") {
      return;
    }

    initializeJobboardResults({
      hasJobs: resultConfig.dataset.hasJobs === "true",
      currentPageContent: resultConfig.dataset.currentPageContent || "",
    });

    resultConfig.dataset.jobboardResultsBound = "true";
  });

  if (document.__gitjobsJobboardResultsLifecycleBound) {
    return;
  }

  document.addEventListener("htmx:afterSwap", (event) => {
    initializeJobboardResultsFromDom(event.target);
  });
  document.addEventListener("htmx:historyRestore", () => {
    initializeJobboardResultsFromDom(document);
  });
  window.addEventListener("pageshow", () => {
    initializeJobboardResultsFromDom(document);
  });

  document.__gitjobsJobboardResultsLifecycleBound = true;
};

initializeJobboardResultsFromDom();
