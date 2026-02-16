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
 * Checks if the current path is a dashboard route.
 * @returns {boolean} True when on a dashboard page
 */
export const isDashboardPath = () => {
  const path = window?.location?.pathname || "";
  return path.startsWith("/dashboard");
};

/**
 * Scrolls to the top of the dashboard so alerts stay visible.
 */
export const scrollToDashboardTop = () => {
  if (!isDashboardPath() || typeof window?.scrollTo !== "function") {
    return;
  }

  window.scrollTo({ top: 0, behavior: "auto" });
};

/**
 * Copies text to clipboard using Clipboard API with a textarea fallback.
 * @param {string} content - Text content to copy
 * @returns {Promise<void>} Resolves when content is copied
 */
export const copyToClipboard = async (content) => {
  if (navigator.clipboard?.writeText) {
    await navigator.clipboard.writeText(content);
    return;
  }

  const temporaryInput = document.createElement("textarea");
  temporaryInput.value = content;
  temporaryInput.setAttribute("readonly", "");
  temporaryInput.style.position = "absolute";
  temporaryInput.style.left = "-9999px";
  document.body.appendChild(temporaryInput);
  temporaryInput.select();
  const copied = document.execCommand("copy");
  document.body.removeChild(temporaryInput);
  if (!copied) {
    throw new Error("Failed to copy text.");
  }
};

/**
 * Ensures a DOM element has an id by generating one when missing.
 * @param {Object} options - Id generation options
 * @param {HTMLElement} options.element - Element that needs an id
 * @param {string} options.prefix - Prefix for generated ids
 * @param {string} options.counterKey - Document key storing the incrementing counter
 * @returns {string} Existing or generated element id
 */
export const ensureElementId = ({ element, prefix, counterKey }) => {
  if (!element) {
    return "";
  }

  if (element.id) {
    return element.id;
  }

  const currentCounter = Number.parseInt(document[counterKey] || "0", 10);
  const nextCounter = Number.isNaN(currentCounter) ? 1 : currentCounter + 1;
  document[counterKey] = String(nextCounter);
  element.id = `${prefix}-${nextCounter}`;

  return element.id;
};

/**
 * Checks whether an element is fully visible in the viewport.
 * @param {HTMLElement} element - Element to check
 * @returns {boolean} True if element is fully visible
 */
export const isElementInView = (element) => {
  if (!element || typeof element.getBoundingClientRect !== "function") {
    return true;
  }

  const rect = element.getBoundingClientRect();
  const viewHeight = window.innerHeight || document.documentElement.clientHeight;
  const viewWidth = window.innerWidth || document.documentElement.clientWidth;

  return rect.top >= 0 && rect.left >= 0 && rect.bottom <= viewHeight && rect.right <= viewWidth;
};

/**
 * Sets drawer visibility, backdrop state, and body scroll lock.
 * Uses drawer open-state transitions to avoid lock counter drift.
 * @param {Object} options - Drawer visibility options
 * @param {string} options.drawerId - Drawer element id
 * @param {boolean} options.open - Whether drawer should be open
 * @param {string} [options.backdropId="drawer-backdrop"] - Backdrop element id
 */
export const setDrawerVisibility = ({ drawerId, open, backdropId = "drawer-backdrop" }) => {
  const drawer = document.getElementById(drawerId);
  const wasOpen = drawer?.dataset.open === "true";
  if (drawer) {
    if (open) {
      drawer.classList.add("transition-transform");
      drawer.classList.remove("-translate-x-full");
      drawer.dataset.open = "true";
    } else {
      drawer.classList.add("-translate-x-full");
      drawer.classList.remove("transition-transform");
      drawer.dataset.open = "false";
      drawer.scrollTop = 0;
    }

    drawer.setAttribute("aria-hidden", open ? "false" : "true");

    if (open && !wasOpen) {
      lockBodyScroll();
    }

    if (!open && wasOpen) {
      unlockBodyScroll();
    }
  }

  const backdrop = document.getElementById(backdropId);
  if (!backdrop) {
    return;
  }

  if (open) {
    backdrop.classList.remove("hidden");
  } else {
    backdrop.classList.add("hidden");
  }
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
 * Initializes click handlers that close a modal.
 * @param {Object} options - Close handler options
 * @param {string} [options.modalId] - The target modal id
 * @param {string[]} options.triggerIds - Element ids that trigger modal close
 * @param {Function} [options.onClose] - Optional callback before closing
 * @param {Function} [options.closeHandler] - Optional custom close function
 * @param {boolean} [options.closeOnEscape=true] - Close modal when Escape is pressed
 */
export const initializeModalCloseHandlers = ({
  modalId,
  triggerIds,
  onClose,
  closeHandler,
  closeOnEscape = true,
} = {}) => {
  if (!Array.isArray(triggerIds) || triggerIds.length === 0) {
    return;
  }

  if (!modalId && typeof closeHandler !== "function") {
    return;
  }

  const isModalOpen = (modalElement) => {
    if (!modalElement) {
      return false;
    }

    if (modalElement.dataset.open === "true") {
      return true;
    }

    return !modalElement.classList.contains("hidden");
  };

  const closeModal = () => {
    if (typeof onClose === "function") {
      onClose();
    }

    if (typeof closeHandler === "function") {
      closeHandler();
      return;
    }

    if (modalId) {
      toggleModalVisibility(modalId, "close");
    }
  };

  triggerIds.forEach((triggerId) => {
    const trigger = document.getElementById(triggerId);
    if (!trigger || trigger.dataset.modalCloseBound === "true") {
      return;
    }

    trigger.addEventListener("click", closeModal);
    trigger.dataset.modalCloseBound = "true";
  });

  if (closeOnEscape && modalId) {
    const escapeGuardKey = `__gitjobsModalEscapeBound:${modalId}`;
    if (!document[escapeGuardKey]) {
      document.addEventListener("keydown", (event) => {
        if (event.key !== "Escape" || event.__gitjobsModalClosed) {
          return;
        }

        const modal = document.getElementById(modalId);
        if (!isModalOpen(modal)) {
          return;
        }

        closeModal();
        event.__gitjobsModalClosed = true;
      });

      document[escapeGuardKey] = true;
    }
  }
};

/**
 * Initializes preview modal close handlers for backdrop and close button.
 * @param {Object} [options] - Close behavior options
 * @param {boolean} [options.cleanJobIdParam=false] - Remove job_id query param
 * @param {Function} [options.onClose] - Optional callback executed before close
 */
export const initializePreviewModalCloseHandlers = ({ cleanJobIdParam = false, onClose } = {}) => {
  initializeModalCloseHandlers({
    modalId: "preview-modal",
    triggerIds: ["backdrop-preview-modal", "close-preview-modal"],
    onClose: () => {
      const previewContent = document.getElementById("preview-content");
      if (previewContent) {
        previewContent.scrollTop = 0;
      }

      if (cleanJobIdParam) {
        removeParamFromQueryString("job_id", {
          modal_preview: false,
        });
      }

      if (typeof onClose === "function") {
        onClose();
      }
    },
  });
};

/**
 * Initializes dropdown lifecycle for a button and menu pair.
 * Supports outside-click close, Escape close, and duplicate-listener guards.
 * @param {Object} options - Dropdown initialization options
 * @param {string} options.buttonId - Trigger button element id
 * @param {string} options.dropdownId - Dropdown menu element id
 * @param {string} options.guardKey - Document key used to bind global listeners once
 * @param {string} [options.closeOnItemClickSelector] - Optional selector to close on item click
 * @returns {Function} Function that closes the dropdown
 */
export const initializeButtonDropdown = ({
  buttonId,
  dropdownId,
  guardKey,
  closeOnItemClickSelector = "",
}) => {
  const hideDropdown = () => {
    const currentButton = document.getElementById(buttonId);
    const currentDropdown = document.getElementById(dropdownId);
    if (!currentDropdown) {
      return;
    }

    currentDropdown.classList.add("hidden");
    currentDropdown.setAttribute("aria-hidden", "true");
    if (currentButton) {
      currentButton.setAttribute("aria-expanded", "false");
    }
  };

  const button = document.getElementById(buttonId);
  const dropdown = document.getElementById(dropdownId);
  const buttonGuardKey = `__gitjobsDropdownTriggerBound:${buttonId}:${dropdownId}`;
  if (button && dropdown && button[buttonGuardKey] !== true) {
    dropdown.setAttribute("aria-hidden", dropdown.classList.contains("hidden") ? "true" : "false");

    button.addEventListener("click", (event) => {
      event.stopPropagation();
      const currentButton = document.getElementById(buttonId);
      const currentDropdown = document.getElementById(dropdownId);
      if (!currentButton || !currentDropdown) {
        return;
      }

      const willOpen = currentDropdown.classList.contains("hidden");
      if (willOpen) {
        currentDropdown.classList.remove("hidden");
        currentDropdown.setAttribute("aria-hidden", "false");
      } else {
        currentDropdown.classList.add("hidden");
        currentDropdown.setAttribute("aria-hidden", "true");
      }
      currentButton.setAttribute("aria-expanded", willOpen ? "true" : "false");
    });

    button[buttonGuardKey] = true;
  }

  const closeOnItemGuardKey = `__gitjobsDropdownCloseOnItemBound:${buttonId}:${dropdownId}`;
  if (closeOnItemClickSelector && !document[closeOnItemGuardKey]) {
    document.addEventListener(
      "click",
      (event) => {
        const currentDropdown = document.getElementById(dropdownId);
        if (!currentDropdown || currentDropdown.classList.contains("hidden")) {
          return;
        }

        const actionItem = event.target.closest(closeOnItemClickSelector);
        if (!actionItem || !currentDropdown.contains(actionItem)) {
          return;
        }

        if (actionItem.querySelector(".hx-spinner")) {
          return;
        }

        hideDropdown();
      },
      true,
    );

    document[closeOnItemGuardKey] = true;
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
 * @returns {Function & {cancel: Function}} Debounced function with cancel()
 */
export const debounce = (func, timeout = 300) => {
  let timer;
  const debounced = (...args) => {
    clearTimeout(timer);
    timer = setTimeout(() => {
      func.apply(this, args);
    }, timeout);
  };
  debounced.cancel = () => {
    clearTimeout(timer);
  };

  return debounced;
};

/**
 * Registers a debounced resize handler for ECharts instances by element id.
 * Uses an optional guard key to bind only once.
 * @param {Object} options - Resize handler options
 * @param {string[]} options.chartIds - Chart container element ids
 * @param {string} [options.guardKey] - Document flag key for one-time binding
 * @param {number} [options.delay=150] - Debounce delay in milliseconds
 */
export const registerChartResizeHandler = ({ chartIds, guardKey, delay = 150 }) => {
  if (!Array.isArray(chartIds) || chartIds.length === 0) {
    return;
  }

  if (guardKey && document[guardKey]) {
    return;
  }

  const resizeCharts = debounce(() => {
    const echartsApi = window.echarts;
    if (!echartsApi || typeof echartsApi.getInstanceByDom !== "function") {
      return;
    }

    chartIds.forEach((chartId) => {
      const chartDom = document.getElementById(chartId);
      if (!chartDom) {
        return;
      }

      const chartInstance = echartsApi.getInstanceByDom(chartDom);
      if (chartInstance) {
        chartInstance.resize();
      }
    });
  }, delay);

  window.addEventListener("resize", resizeCharts);

  if (guardKey) {
    document[guardKey] = true;
  }
};

/**
 * Binds the same HTMX beforeRequest handler to matching elements once.
 * @param {Object} options - Binding options
 * @param {string} options.selector - CSS selector for target elements
 * @param {Function} options.handler - Callback for htmx:beforeRequest
 * @param {string} [options.boundAttribute="htmxBeforeRequestBound"] - dataset guard attribute
 */
export const bindHtmxBeforeRequestOnce = ({
  selector,
  handler,
  boundAttribute = "htmxBeforeRequestBound",
}) => {
  if (!selector || typeof handler !== "function") {
    return;
  }

  const elements = document.querySelectorAll(selector);
  elements.forEach((element) => {
    if (element.dataset[boundAttribute] === "true") {
      return;
    }

    element.addEventListener("htmx:beforeRequest", handler);
    element.dataset[boundAttribute] = "true";
  });
};

/**
 * Binds the same HTMX afterRequest handler to matching elements once.
 * @param {Object} options - Binding options
 * @param {string} options.selector - CSS selector for target elements
 * @param {Function} options.handler - Callback for htmx:afterRequest
 * @param {string} [options.boundAttribute="htmxAfterRequestBound"] - dataset guard attribute
 */
export const bindHtmxAfterRequestOnce = ({ selector, handler, boundAttribute = "htmxAfterRequestBound" }) => {
  if (!selector || typeof handler !== "function") {
    return;
  }

  const elements = document.querySelectorAll(selector);
  elements.forEach((element) => {
    if (element.dataset[boundAttribute] === "true") {
      return;
    }

    element.addEventListener("htmx:afterRequest", handler);
    element.dataset[boundAttribute] = "true";
  });
};

/**
 * Registers the HTMX extension that strips empty values from requests.
 * Filters blank and "0" values from both GET and non-GET submissions.
 */
export const initializeNoEmptyValuesExtension = () => {
  const htmxInstance = window.htmx;
  if (window.__gitjobsNoEmptyValsRegistered || typeof htmxInstance?.defineExtension !== "function") {
    return;
  }

  const removeEmptyValues = (source) => {
    const filteredEntries = [];

    for (const [key, rawValue] of source.entries()) {
      const value = typeof rawValue === "string" ? rawValue.trim() : String(rawValue);
      if (value === "" || value === "0") {
        continue;
      }

      filteredEntries.push([key, typeof rawValue === "string" ? value : rawValue]);
    }

    return filteredEntries;
  };

  htmxInstance.defineExtension("no-empty-vals", {
    onEvent: (name, event) => {
      if (name !== "htmx:configRequest") {
        return true;
      }

      const request = event.detail;
      if (request.verb !== "get" || !request.useUrlParams) {
        return true;
      }

      const filteredParameters = new FormData();
      for (const [key, value] of removeEmptyValues(request.formData)) {
        filteredParameters.append(key, value);
      }

      request.formData = filteredParameters;
      request.parameters = filteredParameters;

      return true;
    },
    encodeParameters: (xhr, parameters, elt) => {
      const filteredEntries = removeEmptyValues(parameters);

      for (const key of [...parameters.keys()]) {
        parameters.delete(key);
      }

      for (const [key, value] of filteredEntries) {
        parameters.append(key, value);
      }

      return null;
    },
  });

  window.__gitjobsNoEmptyValsRegistered = true;
};

/**
 * Triggers an HTMX action on a form element.
 * @param {string} formId - The ID of the form element
 * @param {string} action - The action to trigger
 */
export const triggerActionOnForm = (formId, action) => {
  const form = document.getElementById(formId);
  if (form && typeof htmx?.trigger === "function") {
    htmx.trigger(form, action);
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

    const dropdownUser = document.getElementById("dropdown-user") || document.getElementById("user-dropdown");
    if (dropdownUser !== null && !dropdownUser.classList.contains("hidden")) {
      dropdownUser.classList.add("hidden");
      dropdownUser.setAttribute("aria-hidden", "true");
    }

    const userDropdownButton = document.getElementById("user-dropdown-button");
    if (userDropdownButton) {
      userDropdownButton.setAttribute("aria-expanded", "false");
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
      setDrawerVisibility({ drawerId: "drawer-menu", open: false });
    }

    if (window.Osano?.cm?.showDrawer) {
      window.Osano.cm.showDrawer("osano-cm-dom-info-dialog-open");
    }
  });

  cookieButton.dataset.cookieBound = "true";
};

/**
 * Binds a toggle checkbox to its hidden input mirror value.
 * @param {Object} options - Toggle binding options
 * @param {string} options.toggleId - Toggle checkbox id
 * @param {string} options.hiddenInputId - Hidden input id
 */
export const bindToggleCheckbox = ({ toggleId, hiddenInputId }) => {
  const toggleCheckbox = document.getElementById(toggleId);
  const hiddenInput = document.getElementById(hiddenInputId);
  if (!toggleCheckbox || !hiddenInput || toggleCheckbox.dataset.toggleMirrorBound === "true") {
    return;
  }

  toggleCheckbox.addEventListener("change", () => {
    hiddenInput.value = toggleCheckbox.checked;
  });
  toggleCheckbox.dataset.toggleMirrorBound = "true";
};

/**
 * Tracks a view for a specific job by sending a POST request.
 * Silently handles errors without user notification.
 * @param {string} jobId - The ID of the job to register a view for
 */
export const trackerJobView = async (jobId) => {
  if (!jobId) return;

  try {
    await fetch(`/jobs/${jobId}/views`, {
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
