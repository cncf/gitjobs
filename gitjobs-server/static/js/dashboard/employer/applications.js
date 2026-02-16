import { initializePreviewButtons } from "/static/js/common/alerts.js";
import { initializeButtonDropdown } from "/static/js/common/common.js";

/**
 * Initializes employer applications list interactions.
 */
export const initializeEmployerApplicationsList = () => {
  const hideJobsDropdown = initializeButtonDropdown({
    buttonId: "jobs-btn",
    dropdownId: "dropdown-jobs",
    guardKey: "__gitjobsApplicationsDropdownBound",
  });

  const dropdownJobButtons = document.querySelectorAll("#dropdown-jobs button");
  dropdownJobButtons.forEach((button) => {
    if (button.dataset.dropdownJobBound === "true") {
      return;
    }

    button.addEventListener("htmx:beforeRequest", hideJobsDropdown);
    button.dataset.dropdownJobBound = "true";
  });

  initializePreviewButtons({
    errorMessage: "Something went wrong previewing the data. Please try again later.",
  });
};
