import { initializeButtonDropdown } from "/static/js/common/common.js";
import { initializeDashboardActionButton } from "/static/js/dashboard/employer/dashboard-actions.js";

/**
 * Initializes employer dashboard home interactions.
 */
export const initializeEmployerHome = () => {
  const hideEmployersDropdown = initializeButtonDropdown({
    buttonId: "employer-btn",
    dropdownId: "dropdown-employers",
    guardKey: "__gitjobsEmployersDropdownBound",
  });

  const dropdownEmployerButtons = document.querySelectorAll("button.employer-button");
  dropdownEmployerButtons.forEach((button) => {
    if (button.dataset.employerDropdownBound === "true") {
      return;
    }

    button.addEventListener("htmx:beforeRequest", hideEmployersDropdown);
    button.dataset.employerDropdownBound = "true";
  });

  initializeDashboardActionButton({
    buttonId: "add-employer-button",
    errorMessage: "Something went wrong loading the employer form. Please try again later.",
    pushStateTitle: "Employer",
    pushStateUrl: "/dashboard/employer",
  });
};
