// Display the active section
export const displayActiveSection = (section) => {
  const btn = document.querySelector(`[data-section=${section}]`);
  const isActive = btn.getAttribute("data-active");
  if (isActive === "false") {
    const buttons = document.querySelectorAll("[data-section]");
    buttons.forEach((secB) => {
      secB.setAttribute("data-active", "false");
      secB.classList.remove("active");
    });
    btn.setAttribute("data-active", "true");
    btn.classList.add("active");

    const sections = document.querySelectorAll("[data-content]");
    sections.forEach((content) => {
      if (content.getAttribute("data-content") !== section) {
        content.classList.add("hidden");
      } else {
        content.classList.remove("hidden");
      }
    });
  }
};

// Validate all forms
export const validateFormData = () => {
  // List of forms to validate
  const FORMS = ["profile", "experience", "education", "projects"];

  // Validate each form
  for (const form of FORMS) {
    const formElement = document.getElementById(`${form}-form`);
    // If the form is not valid, show the section and return false
    if (!formElement.checkValidity()) {
      displayActiveSection(form);
      formElement.reportValidity();
      return false;
    }
  }

  return true;
};
