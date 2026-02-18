import { initializeButtonDropdown } from "/static/js/common/common.js";
import { initializeDashboardActionButton } from "/static/js/dashboard/employer/dashboard-actions.js";

/**
 * Initializes employer dashboard home interactions.
 */
export const initializeEmployerHome = () => {
  initializeButtonDropdown({
    buttonId: "employer-btn",
    dropdownId: "dropdown-employers",
    guardKey: "__gitjobsEmployersDropdownBound",
    closeOnItemClickSelector: "button",
  });

  initializeDashboardActionButton({
    buttonId: "add-employer-button",
    errorMessage: "Something went wrong loading the employer form. Please try again later.",
    pushStateTitle: "Employer",
    pushStateUrl: "/dashboard/employer",
  });
};
