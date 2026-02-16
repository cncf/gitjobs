import { setDrawerVisibility, triggerActionOnForm as triggerFormAction } from "/static/js/common/common.js";

/**
 * Opens the mobile filters drawer.
 * Adds transition effects and manages backdrop visibility.
 */
export const openFiltersDrawer = () => {
  setDrawerVisibility({ drawerId: "drawer-filters", open: true });
};

/**
 * Closes the mobile filters drawer.
 * Removes transition effects and resets scroll position.
 */
export const closeFiltersDrawer = () => {
  setDrawerVisibility({ drawerId: "drawer-filters", open: false });
};

/**
 * Triggers an action on the specified form.
 * @param {string} formId - The ID of the form element
 * @param {string} action - The action to trigger
 * @param {boolean} [fromSearch] - Whether triggered from search input
 */
export const triggerActionOnForm = (formId, action, fromSearch) => {
  // Prevent empty search submissions
  if (fromSearch) {
    const searchInput = document.getElementById("searchbar");
    if (searchInput.value === "") {
      return;
    }
  }

  triggerFormAction(formId, action);
};

/**
 * Handles enter key press for search functionality.
 * @param {KeyboardEvent} event - Keyboard event
 * @param {string} [formId] - Form ID to submit, or redirects if not provided
 */
export const searchOnEnter = (event, formId) => {
  if (event.key === "Enter") {
    if (formId) {
      triggerActionOnForm(formId, "submit");
    } else {
      const searchValue = event.currentTarget.value;
      if (searchValue !== "") {
        document.location.href = `/jobs?ts_query=${searchValue}`;
      }
    }
    event.currentTarget.blur();
  }
};

/**
 * Clears an input field and optionally triggers form submission.
 * @param {string} inputId - The ID of the input field
 * @param {string} [formId] - Optional form ID to submit after clearing
 */
export const cleanInputField = (inputId, formId) => {
  const input = document.getElementById(inputId);
  input.value = "";

  if (formId) {
    triggerActionOnForm(formId, "submit");
  }
};

/**
 * Updates the results section with new content.
 * @param {string} content - HTML content to display
 */
export const updateResults = (content) => {
  const resultsContainer = document.getElementById("results");
  resultsContainer.innerHTML = content;
};

/**
 * Resets all form fields to their default values.
 * Handles various input types and custom components.
 * @param {string} formId - The ID of the form to reset
 */
export const resetForm = async (formId) => {
  const form = document.getElementById(formId);
  if (form) {
    // Reset select elements
    document.querySelectorAll(`select[form=${formId}]`).forEach((selectElement) => {
      if (selectElement.name === "date_range") {
        selectElement.value = "last30-days";
      } else {
        selectElement.value = "";
      }
    });

    // Clear checkboxes and radio buttons
    document
      .querySelectorAll(`input[form=${formId}][type=checkbox]`)
      .forEach((checkbox) => (checkbox.checked = false));
    document
      .querySelectorAll(`input[form=${formId}][type=radio]`)
      .forEach((radio) => (radio.checked = false));

    // Clear text and hidden inputs
    document
      .querySelectorAll(`input[form=${formId}][type=text]`)
      .forEach((textInput) => (textInput.value = ""));
    document
      .querySelectorAll(`input[form=${formId}][type=hidden]`)
      .forEach((hiddenInput) => (hiddenInput.value = ""));

    // Reset custom filter components
    const searchableFilters = document.getElementsByTagName("searchable-filter");
    for (let i = 0; i < searchableFilters.length; i++) {
      await searchableFilters[i].cleanSelected();
    }

    // Reset project search components
    const searchProjects = document.getElementsByTagName("search-projects");
    for (let i = 0; i < searchProjects.length; i++) {
      await searchProjects[i].cleanSelected();
    }

    // Reset range input components
    const rangeFilters = document.getElementsByTagName("input-range");
    for (let i = 0; i < rangeFilters.length; i++) {
      await rangeFilters[i].cleanRange();
    }

    // Reset location search components
    const locationInputs = document.getElementsByTagName("search-location");
    for (let i = 0; i < locationInputs.length; i++) {
      await locationInputs[i].cleanLocation();
    }

    // Clear main search input
    const searchBar = document.getElementById("searchbar");
    if (searchBar) {
      searchBar.value = "";
    }

    // Submit form with cleared values
    triggerActionOnForm(formId, "submit");
  }
};
