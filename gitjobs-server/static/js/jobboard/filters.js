// Open filters view (only for mobile).
export const open = () => {
  const drawer = document.getElementById("drawer-filters");
  drawer.classList.add("transition-transform");
  drawer.classList.remove("-translate-x-full");
  drawer.dataset.open = "true";
  const backdrop = document.getElementById("drawer-backdrop");
  backdrop.classList.remove("hidden");
};

// Close filters view (only for mobile).
export const close = () => {
  const drawer = document.getElementById("drawer-filters");
  drawer.classList.add("-translate-x-full");
  drawer.classList.remove("transition-transform");
  drawer.dataset.open = "false";
  drawer.scrollTop = 0;
  const backdrop = document.getElementById("drawer-backdrop");
  backdrop.classList.add("hidden");
};

// Trigger action on the form provided.
export const triggerActionOnForm = (formId, action, fromSearch) => {
  // Prevent form submission if the search input is empty, and it is triggered
  // from the search input
  if (fromSearch) {
    const input = document.getElementById("searchbar");
    if (input.value === "") {
      return;
    }
  }

  const form = document.getElementById(formId);
  if (form) {
    htmx.trigger(form, action);
  }
};

// Search on enter key press.
export const searchOnEnter = (e, formId) => {
  if (e.key === "Enter") {
    if (formId) {
      triggerActionOnForm(formId, "submit");
    } else {
      const value = e.currentTarget.value;
      if (value !== "") {
        document.location.href = `/jobs?ts_query=${value}`;
      }
    }
    e.currentTarget.blur();
  }
};

// Clean input field and trigger change on form.
export const cleanInputField = (id, formId) => {
  const input = document.getElementById(id);
  input.value = "";

  if (formId) {
    triggerActionOnForm(formId, "submit");
  }
};

// Update results on DOM with the given content.
export const updateResults = (content) => {
  const results = document.getElementById("results");
  results.innerHTML = content;
};

// Reset form.
export const resetForm = async (formId) => {
  const form = document.getElementById(formId);
  if (form) {
    // Clean selects
    document.querySelectorAll(`select[form=${formId}]`).forEach((el) => {
      if (el.name === "date_range") {
        el.value = "last30-days";
      } else {
        el.value = "";
      }
    });

    // Clean radio/checkbox input fields
    document.querySelectorAll(`input[form=${formId}][type=checkbox]`).forEach((el) => (el.checked = false));
    document.querySelectorAll(`input[form=${formId}][type=radio]`).forEach((el) => (el.checked = false));

    // Clean text input fields
    document.querySelectorAll(`input[form=${formId}][type=text]`).forEach((el) => (el.value = ""));
    document.querySelectorAll(`input[form=${formId}][type=hidden]`).forEach((el) => (el.value = ""));

    // Clean selected options in collapsible filters
    const searchableFilters = document.getElementsByTagName("searchable-filter");
    for (let i = 0; i < searchableFilters.length; i++) {
      await searchableFilters[i].cleanSelected();
    }

    // Clean search projects input field
    const searchProjects = document.getElementsByTagName("search-projects");
    for (let i = 0; i < searchProjects.length; i++) {
      await searchProjects[i].cleanSelected();
    }

    // Clean range inputs
    const rangeFilters = document.getElementsByTagName("input-range");
    for (let i = 0; i < rangeFilters.length; i++) {
      await rangeFilters[i].cleanRange();
    }

    // Clean search location input field
    const locationInputs = document.getElementsByTagName("search-location");
    for (let i = 0; i < locationInputs.length; i++) {
      await locationInputs[i].cleanLocation();
    }

    // Clean ts_query input field
    const input = document.getElementById("searchbar");
    if (input) {
      input.value = "";
    }

    // Trigger change on form
    triggerActionOnForm(formId, "submit");
  }
};
