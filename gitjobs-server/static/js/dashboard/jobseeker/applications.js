import { initializeConfirmHtmxButtons, initializePreviewButtons } from "/static/js/common/alerts.js";

/**
 * Initializes job seeker applications list interactions.
 */
export const initializeJobSeekerApplicationsList = () => {
  initializePreviewButtons({
    errorMessage: "Something went wrong previewing the data. Please try again later.",
  });

  initializeConfirmHtmxButtons({
    selector: "[data-cancel-application-button]",
    confirmMessage: "Are you sure you wish to cancel this application?",
    successMessage: "You have successfully canceled the application.",
    errorMessage: "An error occurred canceling this application. Please try again later.",
  });
};
