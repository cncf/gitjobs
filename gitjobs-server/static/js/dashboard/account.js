import { handleHtmxResponse } from "/static/js/common/alerts.js";

/**
 * Initializes HTMX response handling for account update forms.
 */
export const initializeAccountUpdateForms = () => {
  const userDetailsForm = document.getElementById("user-details-form");
  const passwordForm = document.getElementById("password-form");

  if (userDetailsForm && userDetailsForm.dataset.accountDetailsBound !== "true") {
    userDetailsForm.addEventListener("htmx:afterRequest", (event) => {
      handleHtmxResponse({
        xhr: event.detail.xhr,
        errorMessage: "Something went wrong updating the user details. Please try again later.",
      });
    });
    userDetailsForm.dataset.accountDetailsBound = "true";
  }

  if (passwordForm && passwordForm.dataset.accountPasswordBound !== "true") {
    passwordForm.addEventListener("htmx:afterRequest", (event) => {
      handleHtmxResponse({
        xhr: event.detail.xhr,
        errorMessage: "Something went wrong updating the password. Please try again later.",
      });
    });
    passwordForm.dataset.accountPasswordBound = "true";
  }
};
