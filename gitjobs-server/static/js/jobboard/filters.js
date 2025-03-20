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
  const backdrop = document.getElementById("drawer-backdrop");
  backdrop.classList.add("hidden");
};

// Trigger action on the form provided.
export const triggerActionOnForm = (formId, action, fromSearch) => {
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
