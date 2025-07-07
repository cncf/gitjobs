/**
 * Shows or hides a modal by ID.
 * Controls body scroll behavior when modal is open/closed.
 * @param {string} modalId - The ID of the modal element
 * @param {'open'|'close'} status - Whether to open or close the modal
 */
export const toggleModalVisibility = (modalId, status) => {
  const modal = document.getElementById(modalId);
  if (status === "open" && modal) {
    modal.classList.remove("hidden");
    // This is used to hide body scroll when the modal is open
    modal.dataset.open = "true";
  } else if (status === "close") {
    modal.classList.add("hidden");
    // This is used to show body scroll when the modal is open
    modal.dataset.open = "false";
  }
};

/**
 * Creates a debounced version of a function that delays execution.
 * Useful for limiting API calls on user input.
 * @param {Function} func - The function to debounce
 * @param {number} [timeout=300] - Delay in milliseconds
 * @returns {Function} The debounced function
 */
export const debounce = (func, timeout = 300) => {
  let timer;
  return (...args) => {
    clearTimeout(timer);
    timer = setTimeout(() => {
      func.apply(this, args);
    }, timeout);
  };
};

/**
 * Updates an element's HTMX URL attribute by replacing placeholders.
 * Processes the element to enable HTMX functionality.
 * @param {string} elementId - The ID of the element to update
 * @param {string} method - The HTTP method (get, post, etc.)
 * @param {string} data - The value to replace in the URL
 */
export const processNewHtmxUrl = (elementId, method, data) => {
  const element = document.getElementById(elementId);
  if (element) {
    const url = element.dataset.url;
    if (url) {
      const newUrl = url.replace(`{:${element.dataset.replacement}}`, data);
      element.setAttribute(`hx-${method}`, newUrl);
      // Process new URL
      htmx.process(`#${elementId}`);
    }
  }
};

/**
 * Checks if an HTTP status code indicates success (2xx range).
 * @param {number} status - The HTTP status code
 * @returns {boolean} True if status is between 200-299
 */
export const isSuccessfulXHRStatus = (status) => {
  if (status >= 200 && status < 300) {
    return true;
  } else {
    return false;
  }
};

/**
 * Checks if an object is empty after removing the 'id' property.
 * @param {Object} obj - The object to check
 * @returns {boolean} True if all properties (except id) are null/empty/undefined
 */
export const isObjectEmpty = (obj) => {
  // Remove the id key from the object
  const objectWithoutId = { ...obj };
  delete objectWithoutId.id;
  return Object.values(objectWithoutId).every((x) => x === null || x === "" || typeof x === "undefined");
};

/**
 * Converts hyphenated text to space-separated text.
 * Example: "hello-world" becomes "hello world"
 * @param {string} text - The text to unnormalize
 * @returns {string} The unnormalized text
 */
export const unnormalize = (text) => {
  return text.replace(/-/g, " ");
};

/**
 * Adds or updates a parameter in the URL query string.
 * @param {string} param - The parameter name
 * @param {string} value - The parameter value
 * @param {Object} [state] - Optional history state object
 */
export const addParamToQueryString = (param, value, state) => {
  const searchParams = new URLSearchParams(window.location.search);
  if (searchParams.has(param)) {
    searchParams.delete(param);
  }
  searchParams.set(param, value);
  modifyCurrentUrl(searchParams.toString(), state);
};

/**
 * Removes a parameter from the URL query string.
 * @param {string} param - The parameter name to remove
 * @param {Object} [state] - Optional history state object
 */
export const removeParamFromQueryString = (param, state) => {
  const searchParams = new URLSearchParams(window.location.search);
  if (searchParams.has(param)) {
    searchParams.delete(param);
    modifyCurrentUrl(searchParams.toString(), state);
  }
};

/**
 * Gets a parameter value from the URL query string.
 * @param {string} param - The parameter name
 * @returns {string|null} The parameter value or null if not found
 */
export const getParamFromQueryString = (param) => {
  const searchParams = new URLSearchParams(window.location.search);
  return searchParams.get(param);
};

/**
 * Updates the current URL with new parameters without page reload.
 * @param {string} params - The query string parameters
 * @param {Object} [state] - Optional history state object
 */
export const modifyCurrentUrl = (params, state) => {
  const newUrl = `${window.location.pathname}${params ? `?${params}` : ""}`;
  history.pushState(state || {}, "new_url", newUrl);
};

/**
 * Checks for job_id in URL and opens the job preview modal if found.
 * Handles both initial page load and browser back/forward navigation.
 * @param {boolean} [onLoad=false] - True if called on page load (registers view)
 */
export const shouldDisplayJobModal = (onLoad = false) => {
  const jobId = getParamFromQueryString("job_id");
  if (jobId) {
    const elementId = `job-preview-${jobId}`;
    const jobPreviewButton = document.getElementById(elementId);
    if (jobPreviewButton) {
      htmx.process(jobPreviewButton);
      if (onLoad) {
        // Page load: trigger with open-modal event (registers view)
        htmx.trigger(jobPreviewButton, "open-modal");
      } else {
        // Browser navigation: trigger without registering view
        htmx.trigger(jobPreviewButton, "open-modal-on-popstate");
      }
    }
  }
};

/**
 * Registers a view for a specific job by sending a POST request.
 * Silently handles errors without user notification.
 * @param {string} jobId - The ID of the job to register a view for
 */
export const registerJobIdView = async (jobId) => {
  try {
    fetch(`/jobs/${jobId}/views`, {
      method: "POST",
    });
  } catch (error) {
    // Silently ignore errors
  }
};

/**
 * Tracks search appearances for multiple jobs by sending job IDs to the server.
 * Used when search results are displayed to track which jobs appeared.
 * @param {string[]} jobIds - Array of job IDs that appeared in search results
 */
export const trackSearchAppearances = async (jobIds) => {
  if (!jobIds || jobIds.length === 0) return;

  try {
    await fetch("/jobs/search-appearances", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify(jobIds),
    });
  } catch (error) {
    // Silently ignore errors
  }
};

const NUMBER_REGEX = /\.0+$|(\.[0-9]*[1-9])0+$/;

/**
 * Converts large numbers into a more readable format using SI unit suffixes.
 * Numbers under 1000 are returned as-is. Larger numbers are converted to use
 * suffixes like 'k' (thousands), 'M' (millions), etc.
 *
 * @param {number} num - The number to format
 * @param {number} [digits=1] - Number of decimal places to show (default: 1)
 * @returns {string|number} Formatted number with suffix, or original number if < 1000
 *
 * @example
 * prettifyNumber(500);        // Returns: 500
 * prettifyNumber(1200);       // Returns: "1.2k"
 * prettifyNumber(1500000);    // Returns: "1.5M"
 * prettifyNumber(1200, 0);    // Returns: "1k" (no decimals)
 * prettifyNumber(1234, 2);    // Returns: "1.23k" (2 decimal places)
 */
export const prettifyNumber = (num, digits = 1) => {
  if (num < 1000) {
    return num;
  }

  const si = [
    { value: 1, symbol: "" },
    { value: 1e3, symbol: "k" },
    { value: 1e6, symbol: "M" },
    { value: 1e9, symbol: "B" },
    { value: 1e12, symbol: "T" },
    { value: 1e15, symbol: "P" },
    { value: 1e18, symbol: "E" },
  ];
  let i;
  for (i = si.length - 1; i > 0; i--) {
    if (num >= si[i].value) {
      break;
    }
  }
  return (num / si[i].value).toFixed(digits).replace(NUMBER_REGEX, "$1") + si[i].symbol;
};
