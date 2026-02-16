import { handleHtmxResponse, showConfirmAlert } from "/static/js/common/alerts.js";

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

/**
 * Initializes confirm + HTMX response handling for action buttons.
 * @param {Object} params - Initialization options
 * @param {string} params.selector - Selector for action buttons
 * @param {string} params.confirmMessage - Confirmation alert message
 * @param {string} params.errorMessage - Error message for failed requests
 * @param {string} [params.confirmText="Yes"] - Confirm button text
 * @param {string} [params.successMessage=""] - Success message for requests
 */
export const initializeConfirmHtmxButtons = ({
  selector,
  confirmMessage,
  errorMessage,
  confirmText = "Yes",
  successMessage = "",
}) => {
  const actionButtons = document.querySelectorAll(selector);
  actionButtons.forEach((button) => {
    if (button.dataset.confirmHtmxBound === "true") {
      return;
    }

    button.addEventListener("click", () => {
      showConfirmAlert(confirmMessage, button.id, confirmText);
    });

    button.addEventListener("htmx:afterRequest", (event) => {
      handleHtmxResponse({
        xhr: event.detail.xhr,
        successMessage,
        errorMessage,
      });
    });

    button.dataset.confirmHtmxBound = "true";
  });
};
