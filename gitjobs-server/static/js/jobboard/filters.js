// Open filters view (only for mobile).
export const open = () => {
  const drawer = document.getElementById("drawer-filters");
  drawer.classList.remove("-translate-x-full");
  const backdrop = document.getElementById("drawer-backdrop");
  backdrop.classList.remove("hidden");
};

// Close filters view (only for mobile).
export const close = () => {
  const drawer = document.getElementById("drawer-filters");
  drawer.classList.add("-translate-x-full");
  const backdrop = document.getElementById("drawer-backdrop");
  backdrop.classList.add("hidden");
};

// Trigger change on the form provided.
export const triggerChangeOnForm = (formId, fromSearch) => {
  // Prevent form submission if the search input is empty, and it is triggered
  // from the search input
  if (fromSearch) {
    const input = document.getElementById("ts_query");
    if (input.value === "") {
      return;
    }
  }

  const form = document.getElementById(formId);
  if (form) {
    htmx.trigger(form, "change");
  }
};

// Search on enter key press.
export const searchOnEnter = (e, formId) => {
  if (e.key === "Enter") {
    if (formId) {
      triggerChangeOnForm(formId);
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
    triggerChangeOnForm(formId);
  }
};

// Reset form.
export const resetForm = (formId) => {
  const form = document.getElementById(formId);
  if (form) {
    // Clean radio/checkbox input fields
    form.querySelectorAll('input[type=checkbox]').forEach((el) => (el.checked = false));
    form.querySelectorAll('input[type=radio]').forEach((el) => (el.checked = false));

    // Clean selected options in collapsible filters
    const collapsibleFilters = form.getElementsByTagName("collapsible-filter");
    for (let i = 0; i < collapsibleFilters.length; i++) {
      collapsibleFilters[i].cleanSelected();
    }

    // Clean ts_query input field
    const input = document.getElementById("ts_query");
    if (input) {
      input.value = "";
    }

    // Trigger change on form
    triggerChangeOnForm(formId);
  }
};

// Update results on DOM with the given content.
export const updateResults = (content) => {
  const results = document.getElementById("results");
  results.innerHTML = content;
};
