import { handleHtmxResponse } from "/static/js/common/alerts.js";
import { bindHtmxAfterRequestOnce } from "/static/js/common/common.js";

const EMPLOYER_FORM_ID = "employer-form";
const EMPLOYER_FORM_SELECTOR = `#${EMPLOYER_FORM_ID}`;

/**
 * Initializes HTMX response handling for employer profile add/update forms.
 * @param {Object} params - Initialization options
 * @param {string} params.errorMessage - Error message for failed requests
 */
export const initializeEmployerProfileForm = ({ errorMessage }) => {
  bindHtmxAfterRequestOnce({
    selector: EMPLOYER_FORM_SELECTOR,
    handler: (event) => {
      // Ignore HTMX requests from nested controls such as location search.
      if (event.detail.elt.id !== EMPLOYER_FORM_ID) {
        return;
      }

      handleHtmxResponse({
        xhr: event.detail.xhr,
        errorMessage,
      });
    },
    boundAttribute: "employerProfileFormBound",
  });
};
