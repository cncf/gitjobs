/**
 * Triggers a custom action on the specified form using htmx.
 *
 * @param {string} formId - The ID of the form element to trigger the action on.
 * @param {string} action - The htmx event/action to trigger (e.g., "submit").
 */
export const triggerActionOnForm = (formId, action) => {
  const formElement = document.getElementById(formId);
  if (formElement) {
    htmx.trigger(formElement, action);
  }
};

/**
 * Validates salary fields before form submission.
 * Sets or removes required attributes based on the selected salary type (exact or range).
 * Ensures only the relevant salary fields are required and clears unused fields.
 */
export const validateSalaryFields = () => {
  const periodSelect = document.querySelector('select[name="salary_period"]');
  const currencySelect = document.querySelector('select[name="salary_currency"]');
  const exactSalaryInput = document.querySelector('input[name="salary"]');
  const minSalaryInput = document.querySelector('input[name="salary_min"]');
  const maxSalaryInput = document.querySelector('input[name="salary_max"]');

  // Remove required attributes from all salary fields
  periodSelect.removeAttribute("required");
  currencySelect.removeAttribute("required");
  exactSalaryInput.removeAttribute("required");
  minSalaryInput.removeAttribute("required");
  maxSalaryInput.removeAttribute("required");

  const selectedSalaryType = document.querySelector('input[name="salary_kind"]:checked');
  // If the salary type is range, clear exact salary and set required for min, max, period, currency
  if (selectedSalaryType.id === "range") {
    exactSalaryInput.value = "";

    if (minSalaryInput.value !== "" || maxSalaryInput.value !== "") {
      minSalaryInput.setAttribute("required", "required");
      maxSalaryInput.setAttribute("required", "required");
      periodSelect.setAttribute("required", "required");
      currencySelect.setAttribute("required", "required");
    }
    // If the salary type is exact, clear min and max and set required for exact, period, currency
  } else {
    minSalaryInput.value = "";
    maxSalaryInput.value = "";

    if (exactSalaryInput.value !== "") {
      exactSalaryInput.setAttribute("required", "required");
      periodSelect.setAttribute("required", "required");
      currencySelect.setAttribute("required", "required");
    }
  }
};

/**
 * Validates the job title input field.
 * Prevents the use of the word "remote" in the job title and suggests using the workplace field.
 *
 * @param {HTMLInputElement} inputElement - The job title input element to validate.
 */
export const validateJobTitle = (inputElement) => {
  inputElement.setCustomValidity("");
  const jobTitle = inputElement.value.trim();
  if (jobTitle.toLowerCase().includes("remote")) {
    inputElement.setCustomValidity(
      "Please use the workplace field to indicate that a job is remote"
    );
  }
};
