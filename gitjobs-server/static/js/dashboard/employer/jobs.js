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
  const selectedSalaryType = document.querySelector('input[name="salary_kind"]:checked');

  // Ensure all fields are present before proceeding
  if (
    !salaryPeriodField ||
    !salaryCurrencyField ||
    !salaryField ||
    !salaryMinField ||
    !salaryMaxField ||
    !selectedSalaryType
  ) {
    return;
  }

  // Clear all required attributes initially
  salaryPeriodField.removeAttribute("required");
  salaryCurrencyField.removeAttribute("required");
  salaryField.removeAttribute("required");
  salaryMinField.removeAttribute("required");
  salaryMaxField.removeAttribute("required");
  salaryMaxField.setCustomValidity(""); // Clear any previous error

  if (selectedSalaryType.id === "range") {
    // Range salary: clear exact value, set requirements for range fields
    salaryField.value = "";

    if (salaryMinField.value !== "" || salaryMaxField.value !== "") {
      // If min and max are set, validate that max is not less than min
      if (
        salaryMaxField.value &&
        salaryMinField.value &&
        parseInt(salaryMaxField.value) < parseInt(salaryMinField.value)
      ) {
        salaryMaxField.setCustomValidity("Maximum salary cannot be less than minimum salary.");

        // Clear error when user interacts with fields
        salaryMaxField.addEventListener("input", () => {
          salaryMaxField.setCustomValidity(""); // Clear error on input
        });
        salaryMinField.addEventListener("input", () => {
          salaryMaxField.setCustomValidity(""); // Clear error on input
        });
      }

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

  const jobsForm = document.getElementById("jobs-form");
  jobsForm.reportValidity(); // Trigger validation on the form
};

/**
 * Validates open source and upstream commitment values.
 * Ensures that upstream commitment is not greater than open source value.
 */
export const checkOpenSourceValues = () => {
  const openSource = document.querySelector('input[name="open_source"]');
  const upstreamCommitment = document.querySelector('input[name="upstream_commitment"]');

  // Ensure both fields are present before proceeding
  if (!openSource || !upstreamCommitment) {
    return;
  }

  // Clear any previous custom validity messages
  upstreamCommitment.setCustomValidity("");

  if (openSource.value && upstreamCommitment.value) {
    // If both fields are filled, validate that upstream commitment is not greater than open source
    if (parseInt(upstreamCommitment.value) > parseInt(openSource.value)) {
      upstreamCommitment.setCustomValidity("Upstream commitment cannot be greater than open source value.");
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
