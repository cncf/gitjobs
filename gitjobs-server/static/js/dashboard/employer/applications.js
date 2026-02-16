import { initializePreviewButtons } from "/static/js/common/alerts.js";
import { bindHtmxBeforeRequestOnce, initializeButtonDropdown } from "/static/js/common/common.js";

/**
 * Initializes employer applications list interactions.
 */
export const initializeEmployerApplicationsList = () => {
  const hideJobsDropdown = initializeButtonDropdown({
    buttonId: "jobs-btn",
    dropdownId: "dropdown-jobs",
    guardKey: "__gitjobsApplicationsDropdownBound",
  });

  bindHtmxBeforeRequestOnce({
    selector: "#dropdown-jobs button",
    handler: hideJobsDropdown,
    boundAttribute: "dropdownJobBound",
  });

  initializePreviewButtons({
    errorMessage: "Something went wrong previewing the data. Please try again later.",
  });
};
