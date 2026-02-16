/**
 * Locks body scroll by setting overflow to hidden. Uses a counter to handle
 * multiple modals. Only locks scroll when the first modal opens.
 */
export const lockBodyScroll = () => {
  const body = document.body;
  const current = Number.parseInt(body.dataset.modalOpenCount || "0", 10);
  const next = Number.isNaN(current) ? 1 : current + 1;
  body.dataset.modalOpenCount = String(next);

  if (next !== 1) {
    return;
  }

  const scrollbarWidth = window.innerWidth - document.documentElement.clientWidth;
  body.dataset.modalOverflow = body.style.overflow || "";
  body.dataset.modalPaddingRight = body.style.paddingRight || "";

  if (scrollbarWidth > 0) {
    const currentPaddingRight = Number.parseFloat(window.getComputedStyle(body).paddingRight || "0");
    body.style.paddingRight = `${currentPaddingRight + scrollbarWidth}px`;
  }

  body.style.overflow = "hidden";
};

/**
 * Unlocks body scroll by restoring overflow. Uses a counter to handle multiple
 * modals. Only unlocks scroll when all modals are closed.
 */
export const unlockBodyScroll = () => {
  const body = document.body;
  const current = Number.parseInt(body.dataset.modalOpenCount || "0", 10);
  const next = Number.isNaN(current) ? 0 : Math.max(0, current - 1);
  body.dataset.modalOpenCount = String(next);

  if (next !== 0) {
    return;
  }

  const previousOverflow = body.dataset.modalOverflow ?? "";
  const previousPaddingRight = body.dataset.modalPaddingRight ?? "";
  body.style.overflow = previousOverflow;
  body.style.paddingRight = previousPaddingRight;
  delete body.dataset.modalOverflow;
  delete body.dataset.modalPaddingRight;
};

/**
 * Shows or hides a modal by ID.
 * @param {string} modalId - The ID of the modal element
 * @param {'open'|'close'} [status] - Whether to open or close the modal
 */
export const toggleModalVisibility = (modalId, status) => {
  const modal = document.getElementById(modalId);
  if (!modal) {
    return;
  }

  const isOpen = !modal.classList.contains("hidden");

  if (status === "open") {
    if (isOpen) {
      modal.dataset.open = "true";
      modal.setAttribute("aria-hidden", "false");
      return;
    }

    modal.classList.remove("hidden");
    modal.dataset.open = "true";
    modal.setAttribute("aria-hidden", "false");
    lockBodyScroll();
    return;
  }

  if (status === "close") {
    if (isOpen) {
      modal.classList.add("hidden");
      unlockBodyScroll();
    }

    modal.setAttribute("aria-hidden", "true");
    modal.dataset.open = "false";
    return;
  }

  if (isOpen) {
    modal.classList.add("hidden");
    modal.setAttribute("aria-hidden", "true");
    modal.dataset.open = "false";
    unlockBodyScroll();
  } else {
    modal.classList.remove("hidden");
    modal.setAttribute("aria-hidden", "false");
    modal.dataset.open = "true";
    lockBodyScroll();
  }
};

/**
 * Initializes preview modal close handlers for backdrop and close button.
 * @param {Object} [options] - Close behavior options
 * @param {boolean} [options.cleanJobIdParam=false] - Remove job_id query param
 */
export const initializePreviewModalCloseHandlers = ({ cleanJobIdParam = false } = {}) => {
  const onCloseModal = () => {
    const previewContent = document.getElementById("preview-content");
    if (previewContent) {
      previewContent.scrollTop = 0;
    }

    if (cleanJobIdParam) {
      removeParamFromQueryString("job_id", {
        modal_preview: false,
      });
    }
    toggleModalVisibility("preview-modal", "close");
  };

  const backdropPreviewModal = document.getElementById("backdrop-preview-modal");
  if (backdropPreviewModal && backdropPreviewModal.dataset.previewCloseBound !== "true") {
    backdropPreviewModal.addEventListener("click", onCloseModal);
    backdropPreviewModal.dataset.previewCloseBound = "true";
  }

  const closePreviewModal = document.getElementById("close-preview-modal");
  if (closePreviewModal && closePreviewModal.dataset.previewCloseBound !== "true") {
    closePreviewModal.addEventListener("click", onCloseModal);
    closePreviewModal.dataset.previewCloseBound = "true";
  }
};

/**
 * Initializes dropdown lifecycle for a button and menu pair.
 * Supports outside-click close, Escape close, and duplicate-listener guards.
 * @param {Object} options - Dropdown initialization options
 * @param {string} options.buttonId - Trigger button element id
 * @param {string} options.dropdownId - Dropdown menu element id
 * @param {string} options.guardKey - Document key used to bind global listeners once
 * @returns {Function} Function that closes the dropdown
 */
export const initializeButtonDropdown = ({ buttonId, dropdownId, guardKey }) => {
  const hideDropdown = () => {
    const currentButton = document.getElementById(buttonId);
    const currentDropdown = document.getElementById(dropdownId);
    if (!currentDropdown) {
      return;
    }

    currentDropdown.classList.add("hidden");
    if (currentButton) {
      currentButton.setAttribute("aria-expanded", "false");
    }
  };

  const button = document.getElementById(buttonId);
  const dropdown = document.getElementById(dropdownId);
  if (button && dropdown) {
    button.addEventListener("click", (event) => {
      event.stopPropagation();
      const willOpen = dropdown.classList.contains("hidden");
      dropdown.classList.toggle("hidden");
      button.setAttribute("aria-expanded", willOpen ? "true" : "false");
    });
  }

  if (guardKey && !document[guardKey]) {
    document.addEventListener("click", (event) => {
      const currentButton = document.getElementById(buttonId);
      const currentDropdown = document.getElementById(dropdownId);
      if (!currentButton || !currentDropdown || currentDropdown.classList.contains("hidden")) {
        return;
      }

      if (!currentDropdown.contains(event.target) && !currentButton.contains(event.target)) {
        hideDropdown();
      }
    });

    document.addEventListener("keydown", (event) => {
      if (event.key !== "Escape") {
        return;
      }

      const currentButton = document.getElementById(buttonId);
      const currentDropdown = document.getElementById(dropdownId);
      if (!currentButton || !currentDropdown || currentDropdown.classList.contains("hidden")) {
        return;
      }

      hideDropdown();
      currentButton.focus();
    });

    document[guardKey] = true;
  }

  return hideDropdown;
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
 * Initializes global popstate handling for modal and dropdown UI state.
 */
export const initializeGlobalPopstateHandlers = () => {
  if (document.__gitjobsPopstateBound) {
    return;
  }

  window.addEventListener("popstate", (event) => {
    const modalPreview = document.getElementById("preview-modal");
    if (event.state && event.state.modal_preview !== undefined) {
      if (event.state.modal_preview && modalPreview !== null) {
        const jobId = getParamFromQueryString("job_id");
        if (jobId !== modalPreview.dataset.jobId) {
          shouldDisplayJobModal();
        } else {
          toggleModalVisibility("preview-modal", "open");
        }
      } else {
        toggleModalVisibility("preview-modal", "close");
      }
    } else if (modalPreview !== null && modalPreview.dataset.open === "true") {
      toggleModalVisibility("preview-modal", "close");
    }

    const embedCodeModal = document.getElementById("embed-code-modal");
    if (embedCodeModal !== null && embedCodeModal.dataset.open === "true") {
      toggleModalVisibility("embed-code-modal", "close");
    }

    const dropdownUser = document.getElementById("dropdown-user");
    if (dropdownUser !== null && !dropdownUser.classList.contains("hidden")) {
      dropdownUser.classList.add("hidden");
    }
  });

  document.__gitjobsPopstateBound = true;
};

/**
 * Initializes an Osano cookie preferences button.
 * @param {Object} options - Button options
 * @param {string} options.buttonId - Cookie button id
 * @param {boolean} [options.closeDrawer=false] - Close mobile drawer before opening
 */
export const initializeCookiePreferencesButton = ({ buttonId, closeDrawer = false }) => {
  const cookieButton = document.getElementById(buttonId);
  if (!cookieButton || cookieButton.dataset.cookieBound === "true") {
    return;
  }

  cookieButton.addEventListener("click", () => {
    if (closeDrawer) {
      const navigationDrawer = document.getElementById("drawer-menu");
      if (navigationDrawer) {
        navigationDrawer.classList.add("-translate-x-full");
        navigationDrawer.classList.remove("transition-transform");
        navigationDrawer.dataset.open = "false";
        navigationDrawer.scrollTop = 0;
      }

      const backdrop = document.getElementById("drawer-backdrop");
      if (backdrop) {
        backdrop.classList.add("hidden");
      }
    }

    if (window.Osano?.cm?.showDrawer) {
      window.Osano.cm.showDrawer("osano-cm-dom-info-dialog-open");
    }
  });

  cookieButton.dataset.cookieBound = "true";
};

/**
 * Tracks a view for a specific job by sending a POST request.
 * Silently handles errors without user notification.
 * @param {string} jobId - The ID of the job to register a view for
 */
export const trackerJobView = async (jobId) => {
  if (!jobId) return;

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
