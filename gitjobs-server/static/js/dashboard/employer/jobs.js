import { handleHtmxResponse, handlePreviewModalResponse } from "/static/js/common/alerts.js";
import { triggerActionOnForm } from "/static/js/common/common.js";

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
        if (salaryMaxField.dataset.salaryMaxValidationBound !== "true") {
          salaryMaxField.addEventListener("input", () => {
            salaryMaxField.setCustomValidity(""); // Clear error on input
          });
          salaryMaxField.dataset.salaryMaxValidationBound = "true";
        }
        if (salaryMinField.dataset.salaryMinValidationBound !== "true") {
          salaryMinField.addEventListener("input", () => {
            salaryMaxField.setCustomValidity(""); // Clear error on input
          });
          salaryMinField.dataset.salaryMinValidationBound = "true";
        }
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

/**
 * Wires salary kind toggle behavior for fixed/range sections.
 */
export const initializeSalaryKindToggle = () => {
  const salaryOptions = document.querySelectorAll('input[name="salary_kind"]');
  const salaryOptionFixed = document.getElementById("salary_kind_fixed");
  const salaryOptionRange = document.getElementById("salary_kind_range");

  if (!salaryOptions.length || !salaryOptionFixed || !salaryOptionRange) {
    return;
  }

  salaryOptions.forEach((option) => {
    if (option.dataset.salaryKindBound === "true") {
      return;
    }

    option.addEventListener("change", () => {
      if (option.id === "fixed") {
        salaryOptionFixed.classList.remove("hidden");
        salaryOptionRange.classList.add("hidden");
      } else {
        salaryOptionFixed.classList.add("hidden");
        salaryOptionRange.classList.remove("hidden");
      }
    });

    option.dataset.salaryKindBound = "true";
  });
};

/**
 * Initializes shared behavior for employer job add/update forms.
 * @param {Object} options - Form behavior options
 * @param {string} options.successMessage - Success message for save/update requests
 * @param {string} options.errorMessage - Error message for save/update requests
 * @param {string} [options.publishButtonId] - Optional publish button id
 */
export const initializeEmployerJobForm = ({ successMessage, errorMessage, publishButtonId = "" }) => {
  const jobsForm = document.getElementById("jobs-form");
  if (!jobsForm) {
    return;
  }

  const jobTitleInput = document.getElementById("title");
  if (jobTitleInput && jobTitleInput.dataset.jobTitleValidationBound !== "true") {
    jobTitleInput.addEventListener("input", () => {
      checkJobTitle(jobTitleInput);
    });
    jobTitleInput.dataset.jobTitleValidationBound = "true";
  }

  const openSourceInput = document.querySelector('input[name="open_source"]');
  if (openSourceInput && openSourceInput.dataset.openSourceValidationBound !== "true") {
    openSourceInput.addEventListener("input", checkOpenSourceValues);
    openSourceInput.dataset.openSourceValidationBound = "true";
  }

  const upstreamCommitmentInput = document.querySelector('input[name="upstream_commitment"]');
  if (
    upstreamCommitmentInput &&
    upstreamCommitmentInput.dataset.upstreamCommitmentValidationBound !== "true"
  ) {
    upstreamCommitmentInput.addEventListener("input", checkOpenSourceValues);
    upstreamCommitmentInput.dataset.upstreamCommitmentValidationBound = "true";
  }

  if (jobsForm.dataset.jobsFormTriggerBound !== "true") {
    jobsForm.addEventListener("htmx:trigger", () => {
      checkSalaryBeforeSubmit();
    });
    jobsForm.dataset.jobsFormTriggerBound = "true";
  }

  if (jobsForm.dataset.jobsFormAfterRequestBound !== "true") {
    jobsForm.addEventListener("htmx:afterRequest", (event) => {
      if (event.detail.elt.id !== "jobs-form") {
        return;
      }

      jobsForm.setAttribute("hx-indicator", "#dashboard-spinner, #save-spinner");
      handleHtmxResponse({
        xhr: event.detail.xhr,
        successMessage,
        errorMessage,
      });
    });
    jobsForm.dataset.jobsFormAfterRequestBound = "true";
  }

  if (publishButtonId) {
    const publishButton = document.getElementById(publishButtonId);
    if (publishButton && publishButton.dataset.publishJobBound !== "true") {
      publishButton.addEventListener("click", () => {
        jobsForm.setAttribute("hx-indicator", "#dashboard-spinner, #publish-spinner");
        const statusInput = jobsForm.querySelector('input[name="status"]');
        if (statusInput) {
          statusInput.value = "pending-approval";
        }

        if (!jobsForm.checkValidity()) {
          jobsForm.reportValidity();
        } else {
          triggerActionOnForm("jobs-form", "submit");
        }
      });
      publishButton.dataset.publishJobBound = "true";
    }
  }

  const previewButton = document.getElementById("preview-button");
  if (previewButton && previewButton.dataset.previewJobBound !== "true") {
    previewButton.addEventListener("htmx:afterRequest", (event) => {
      handlePreviewModalResponse({
        xhr: event.detail.xhr,
        invalidMessage: "You must fill in all required fields to be able to preview the job.",
        errorMessage: "Something went wrong previewing the data. Please try again later.",
      });
    });
    previewButton.dataset.previewJobBound = "true";
  }
};
