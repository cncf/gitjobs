import { handleHtmxResponse } from "/static/js/common/alerts.js";
import { bindHtmxAfterRequestOnce } from "/static/js/common/common.js";

const USER_DETAILS_FORM_SELECTOR = "#user-details-form";
const PASSWORD_FORM_SELECTOR = "#password-form";

/**
 * Initializes HTMX response handling for account update forms.
 */
export const initializeAccountUpdateForms = () => {
  bindHtmxAfterRequestOnce({
    selector: USER_DETAILS_FORM_SELECTOR,
    handler: (event) => {
      handleHtmxResponse({
        xhr: event.detail.xhr,
        errorMessage: "Something went wrong updating the user details. Please try again later.",
      });
    },
    boundAttribute: "accountDetailsBound",
  });

  bindHtmxAfterRequestOnce({
    selector: PASSWORD_FORM_SELECTOR,
    handler: (event) => {
      handleHtmxResponse({
        xhr: event.detail.xhr,
        errorMessage: "Something went wrong updating the password. Please try again later.",
      });
    },
    boundAttribute: "accountPasswordBound",
  });
};
