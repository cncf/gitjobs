import { handleHtmxResponse } from "/static/js/common/alerts.js";

/**
 * Initializes HTMX response handling for a dashboard action button.
 * @param {Object} params - Initialization options
 * @param {string} params.buttonId - Action button id
 * @param {string} params.errorMessage - Error message for failed requests
 * @param {string} params.pushStateTitle - Browser history title
 * @param {string} params.pushStateUrl - Browser history URL
 */
export const initializeDashboardActionButton = ({ buttonId, errorMessage, pushStateTitle, pushStateUrl }) => {
  const actionButton = document.getElementById(buttonId);
  if (!actionButton) {
    return;
  }

  if (actionButton.dataset.dashboardActionBound === "true") {
    return;
  }

  actionButton.addEventListener("htmx:afterRequest", (event) => {
    if (
      handleHtmxResponse({
        xhr: event.detail.xhr,
        errorMessage,
      })
    ) {
      history.pushState({}, pushStateTitle, pushStateUrl);
    }
  });

  actionButton.dataset.dashboardActionBound = "true";
};
