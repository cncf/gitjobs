import { handleHtmxResponse } from "/static/js/common/alerts.js";
import { bindHtmxAfterRequestOnce } from "/static/js/common/common.js";

/**
 * Initializes HTMX response handling for account update forms.
 */
export const initializeAccountUpdateForms = () => {
  bindHtmxAfterRequestOnce({
    selector: "#user-details-form",
    handler: (event) => {
      handleHtmxResponse({
        xhr: event.detail.xhr,
        errorMessage: "Something went wrong updating the user details. Please try again later.",
      });
    },
    boundAttribute: "accountDetailsBound",
  });

  bindHtmxAfterRequestOnce({
    selector: "#password-form",
    handler: (event) => {
      handleHtmxResponse({
        xhr: event.detail.xhr,
        errorMessage: "Something went wrong updating the password. Please try again later.",
      });
    },
    boundAttribute: "accountPasswordBound",
  });
};
