/**
 * Triggers an HTMX action on a form element.
 * @param {string} formId - The ID of the form element
 * @param {string} action - The action to trigger
 */
export const triggerActionOnForm = (formId, action) => {
  const form = document.getElementById(formId);
  if (form) {
    htmx.trigger(form, action);
  }
};

/**
 * Validates and adjusts salary fields based on selected salary type.
 * Ensures proper required attributes for range vs exact salary.
 */
export const checkSalaryBeforeSubmit = () => {
  const salaryPeriodField = document.querySelector('select[name="salary_period"]');
  const salaryCurrencyField = document.querySelector('select[name="salary_currency"]');
  const salaryField = document.querySelector('input[name="salary"]');
  const salaryMinField = document.querySelector('input[name="salary_min"]');
  const salaryMaxField = document.querySelector('input[name="salary_max"]');

  // Clear all required attributes initially
  salaryPeriodField.removeAttribute("required");
  salaryCurrencyField.removeAttribute("required");
  salaryField.removeAttribute("required");
  salaryMinField.removeAttribute("required");
  salaryMaxField.removeAttribute("required");

  const selectedSalaryType = document.querySelector('input[name="salary_kind"]:checked');

  if (selectedSalaryType.id === "range") {
    // Range salary: clear exact value, set requirements for range fields
    salaryField.value = "";

    if (salaryMinField.value !== "" || salaryMaxField.value !== "") {
      salaryMinField.setAttribute("required", "required");
      salaryMaxField.setAttribute("required", "required");
      salaryPeriodField.setAttribute("required", "required");
      salaryCurrencyField.setAttribute("required", "required");
    }
  } else {
    // Exact salary: clear range values, set requirements for exact fields
    salaryMinField.value = "";
    salaryMaxField.value = "";

    if (salaryField.value !== "") {
      salaryField.setAttribute("required", "required");
      salaryPeriodField.setAttribute("required", "required");
      salaryCurrencyField.setAttribute("required", "required");
    }
  }
};

/**
 * Validates job title to prevent "remote" in title.
 * @param {HTMLInputElement} input - The job title input element
 */
export const checkJobTitle = (input) => {
  input.setCustomValidity("");
  const jobTitle = input.value.trim();
  if (jobTitle.toLowerCase().includes("remote")) {
    input.setCustomValidity("Please use the workplace field to indicate that a job is remote");
  }
};
