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
  console.log("triggerChangeOnForm", formId);
  if (form) {
    htmx.trigger(form, "change");
  }
};
