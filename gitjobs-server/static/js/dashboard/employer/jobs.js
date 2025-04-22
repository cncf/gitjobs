// Trigger action on the form provided.
export const triggerActionOnForm = (formId, action) => {
  const form = document.getElementById(formId);
  if (form) {
    htmx.trigger(form, action);
  }
};

export const checkSalaryBeforeSubmit = () => {
  const salaryPeriod = document.querySelector('select[name="salary_period"]');
  const salaryCurrency = document.querySelector('select[name="salary_currency"]');
  const salary = document.querySelector('input[name="salary"]');
  const salaryMin = document.querySelector('input[name="salary_min"]');
  const salaryMax = document.querySelector('input[name="salary_max"]');

  // Remove required attributes from all salary fields
  salaryPeriod.removeAttribute("required");
  salaryCurrency.removeAttribute("required");
  salary.removeAttribute("required");
  salaryMin.removeAttribute("required");
  salaryMax.removeAttribute("required");

  const selectedSalaryOption = document.querySelector('input[name="salary_kind"]:checked');
  // If the salary option is range, remove the salary value and set correct required attributes
  // for min, max, period and currency
  if (selectedSalaryOption.id === "range") {
    salary.value = "";

    if (salaryMin.value !== "" || salaryMax.value !== "") {
      salaryMin.setAttribute("required", "required");
      salaryMax.setAttribute("required", "required");
      salaryPeriod.setAttribute("required", "required");
      salaryCurrency.setAttribute("required", "required");
    }
    // If the salary option is exact, remove the salary min and max values and set correct required attributes
    // for salary, period and currency
  } else {
    salaryMin.value = "";
    salaryMax.value = "";

    if (salary.value !== "") {
      salary.setAttribute("required", "required");
      salaryPeriod.setAttribute("required", "required");
      salaryCurrency.setAttribute("required", "required");
    }
  }
};

export const checkJobTitle = (input) => {
  input.setCustomValidity("");
  const jobTitle = input.value.trim();
  if (jobTitle.toLowerCase().includes("remote")) {
    input.setCustomValidity("Please use the workplace field to indicate that a job is remote");
  }
};
