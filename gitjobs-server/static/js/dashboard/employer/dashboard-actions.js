import { handleHtmxResponse } from "/static/js/common/alerts.js";
import { bindHtmxAfterRequestOnce } from "/static/js/common/common.js";

/**
 * Initializes HTMX response handling for a dashboard action button.
 * @param {Object} params - Initialization options
 * @param {string} params.buttonId - Action button id
 * @param {string} params.errorMessage - Error message for failed requests
 * @param {string} params.pushStateTitle - Browser history title
 * @param {string} params.pushStateUrl - Browser history URL
 */
export const initializeDashboardActionButton = ({ buttonId, errorMessage, pushStateTitle, pushStateUrl }) => {
  bindHtmxAfterRequestOnce({
    selector: `#${buttonId}`,
    handler: (event) => {
      if (
        handleHtmxResponse({
          xhr: event.detail.xhr,
          errorMessage,
        })
      ) {
        history.pushState({}, pushStateTitle, pushStateUrl);
      }
    },
    boundAttribute: "dashboardActionBound",
  });
};
