/**
 * Displays the specified section and updates navigation state.
 * @param {string} section - The section identifier to display
 */
export const displayActiveSection = (section) => {
  const navigationButton = document.querySelector(`[data-section=${section}]`);
  const isActive = navigationButton.getAttribute("data-active");
  if (isActive === "false" && navigationButton) {
    const allButtons = document.querySelectorAll("[data-section]");
    allButtons.forEach((button) => {
      button.setAttribute("data-active", "false");
      button.classList.remove("active");
    });
    navigationButton.setAttribute("data-active", "true");
    navigationButton.classList.add("active");

    const allSections = document.querySelectorAll("[data-content]");
    allSections.forEach((content) => {
      if (content.getAttribute("data-content") !== section) {
        content.classList.add("hidden");
      } else {
        content.classList.remove("hidden");
      }
    });
  }
};

/**
 * Validates all job seeker profile forms.
 * Shows first invalid section if validation fails.
 * @returns {boolean} True if all forms are valid
 */
export const validateFormData = () => {
  const formSections = ["profile", "experience", "education", "projects"];

  for (const formName of formSections) {
    const formElement = document.getElementById(`${formName}-form`);

    if (!formElement.checkValidity()) {
      displayActiveSection(formName);
      formElement.reportValidity();
      return false;
    }
  }

  return true;
};
