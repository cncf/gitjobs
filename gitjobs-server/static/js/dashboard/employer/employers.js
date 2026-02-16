import { handleHtmxResponse } from "/static/js/common/alerts.js";

/**
 * Initializes HTMX response handling for employer profile add/update forms.
 * @param {Object} params - Initialization options
 * @param {string} params.errorMessage - Error message for failed requests
 * @param {boolean} [params.scrollToTopOnError=false] - Scroll to top after errors
 */
export const initializeEmployerProfileForm = ({ errorMessage, scrollToTopOnError = false }) => {
  const employerForm = document.getElementById("employer-form");
  if (!employerForm) {
    return;
  }

  employerForm.addEventListener("htmx:afterRequest", (event) => {
    // Ignore HTMX requests from nested controls such as location search.
    if (event.detail.elt.id !== "employer-form") {
      return;
    }

    const ok = handleHtmxResponse({
      xhr: event.detail.xhr,
      errorMessage,
    });
    if (!ok && scrollToTopOnError) {
      window.scrollTo(0, 0);
    }
  });
};
