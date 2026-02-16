import { initializePreviewButtons } from "/static/js/common/alerts.js";
import { initializeButtonDropdown } from "/static/js/common/common.js";

/**
 * Initializes employer applications list interactions.
 */
export const initializeEmployerApplicationsList = () => {
  initializeButtonDropdown({
    buttonId: "jobs-btn",
    dropdownId: "dropdown-jobs",
    guardKey: "__gitjobsApplicationsDropdownBound",
    closeOnItemClickSelector: "button",
  });

  initializePreviewButtons({
    errorMessage: "Something went wrong previewing the data. Please try again later.",
  });
};
