/**
 * Sets the visibility of a modal element by its ID.
 * @param {string} modalId - The ID of the modal element.
 * @param {"open"|"close"} visibility - Whether to show ("open") or hide ("close") the modal.
 */
export const setModalVisibility = (modalId, visibility) => {
  const modal = document.getElementById(modalId);
  if (visibility === "open" && modal) {
    modal.classList.remove("hidden");
    // Hide body scroll when the modal is open
    modal.dataset.open = "true";
  } else if (visibility === "close") {
    modal.classList.add("hidden");
    // Show body scroll when the modal is closed
    modal.dataset.open = "false";
  }
};

/**
 * Returns a debounced version of the provided callback.
 * The callback will only be executed after the specified delay has elapsed since the last call.
 * @param {Function} callback - The function to debounce.
 * @param {number} [delay=300] - The debounce delay in milliseconds.
 * @returns {Function} Debounced function.
 */
export const debounce = (callback, delay = 300) => {
  let timer;
  return (...args) => {
    clearTimeout(timer);
    timer = setTimeout(() => {
      callback.apply(this, args);
    }, delay);
  };
};

/**
 * Updates the htmx URL attribute for an element and processes it.
 * @param {string} elementId - The ID of the element to update.
 * @param {string} httpMethod - The HTTP method (e.g., "get", "post") for the hx- attribute.
 * @param {string} replacementValue - The value to replace in the URL template.
 */
export const updateHtmxElementUrl = (elementId, httpMethod, replacementValue) => {
  const element = document.getElementById(elementId);
  if (element) {
    const url = element.dataset.url;
    if (url) {
      const newUrl = url.replace(`{:${element.dataset.replacement}}`, replacementValue);
      element.setAttribute(`hx-${httpMethod}`, newUrl);
      // Process new URL
      htmx.process(`#${elementId}`);
    }
  }
};

/**
 * Checks if an HTTP status code is considered successful (2xx).
 * @param {number} statusCode - The HTTP status code to check.
 * @returns {boolean} True if successful, false otherwise.
 */
export const isHttpStatusSuccessful = (statusCode) => {
  return statusCode >= 200 && statusCode < 300;
};

/**
 * Checks if all values in an object (except 'id') are empty (null, "", or undefined).
 * @param {Object} object - The object to check.
 * @returns {boolean} True if all values are empty, false otherwise.
 */
export const isObjectValuesEmpty = (object) => {
  // Remove the id key from the object
  const objectWithoutId = { ...object };
  delete objectWithoutId.id;
  return Object.values(objectWithoutId).every((x) => x === null || x === "" || x === undefined);
};

/**
 * Converts hyphens in a string to spaces.
 * @param {string} inputText - The text to unnormalize.
 * @returns {string} The unnormalized text.
 */
export const unnormalize = (inputText) => {
  return inputText.replace(/-/g, " ");
};

/**
 * Sets a query parameter in the URL and updates browser history.
 * @param {string} paramName - The name of the query parameter.
 * @param {string} paramValue - The value to set for the parameter.
 * @param {Object} [stateObj] - Optional state object for history.
 */
export const setQueryParam = (paramName, paramValue, stateObj) => {
  const searchParams = new URLSearchParams(window.location.search);
  searchParams.set(paramName, paramValue);
  updateBrowserUrl(searchParams.toString(), stateObj);
};

/**
 * Removes a query parameter from the URL and updates browser history.
 * @param {string} paramName - The name of the query parameter to remove.
 * @param {Object} [stateObj] - Optional state object for history.
 */
export const removeQueryParam = (paramName, stateObj) => {
  const searchParams = new URLSearchParams(window.location.search);
  if (searchParams.has(paramName)) {
    searchParams.delete(paramName);
    updateBrowserUrl(searchParams.toString(), stateObj);
  }
};

/**
 * Retrieves the value of a query parameter from the current URL.
 * @param {string} paramName - The name of the query parameter.
 * @returns {string|null} The value of the parameter, or null if not found.
 */
export const getQueryParam = (paramName) => {
  const searchParams = new URLSearchParams(window.location.search);
  return searchParams.get(paramName);
};

/**
 * Updates the browser's URL with the provided query string and state object.
 * @param {string} queryString - The query string to set in the URL.
 * @param {Object} [stateObj] - Optional state object for history.
 */
export const updateBrowserUrl = (queryString, stateObj) => {
  const newUrl = `${window.location.pathname}${queryString ? `?${queryString}` : ""}`;
  history.pushState(stateObj || {}, "new_url", newUrl);
};

/**
 * Checks if the job preview modal should be displayed based on the URL.
 * If so, processes and triggers the appropriate modal event.
 * @param {boolean} [onPageLoad=false] - Whether this check is on initial page load.
 */
export const shouldShowJobPreviewModal = (onPageLoad = false) => {
  // Check if the job_id parameter is present in the URL
  const job_id = getQueryParam("job_id");
  if (job_id) {
    const elId = `job-preview-${job_id}`;
    // Check if the job preview button exists
    const jobPreviewBtn = document.getElementById(elId);
    if (jobPreviewBtn) {
      // Process the button
      htmx.process(jobPreviewBtn);
      // Open the modal
      if (onPageLoad) {
        // If the page is loaded, trigger the modal with the open-modal event (register view)
        htmx.trigger(jobPreviewBtn, "open-modal");
      } else {
        // If not on page load, trigger the modal with the open-modal-on-popstate event (do not register view)
        htmx.trigger(jobPreviewBtn, "open-modal-on-popstate");
      }
    }
  }
};

/**
 * Registers a view for a job by sending a POST request.
 * @param {string|number} jobId - The ID of the job to register a view for.
 */
export const registerJobView = async (jobId) => {
  try {
    await fetch(`/jobs/${jobId}/views`, {
      method: "POST",
    });
  } catch (error) {
    // Silently ignore errors
  }
};
