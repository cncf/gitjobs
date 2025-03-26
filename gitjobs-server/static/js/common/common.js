// Show or hide the provided modal.
export const toggleModalVisibility = (modalId, status) => {
  const modal = document.getElementById(modalId);
  if (status === "open") {
    modal.classList.remove("hidden");
    // This is used to hide body scroll when the modal is open
    modal.dataset.open = "true";
  } else if (status === "close") {
    modal.classList.add("hidden");
    // This is used to show body scroll when the modal is open
    modal.dataset.open = "false";
  }
};

// Function to delay the execution of a function
export const debounce = (func, timeout = 300) => {
  let timer;
  return (...args) => {
    clearTimeout(timer);
    timer = setTimeout(() => {
      func.apply(this, args);
    }, timeout);
  };
};

// Function to process new URL for htmx
export const processNewHtmxUrl = (idEl, method, data) => {
  const element = document.getElementById(idEl);
  if (element) {
    const url = element.dataset.url;
    if (url) {
      const newUrl = url.replace(`{:${element.dataset.replacement}}`, data);
      element.setAttribute(`hx-${method}`, newUrl);
      // Process new URL
      htmx.process(`#${idEl}`);
    }
  }
};

// Function to check if the status of the XHR request is successful
export const isSuccessfulXHRStatus = (status) => {
  if (status >= 200 && status < 300) {
    return true;
  } else {
    return false;
  }
};

// Function to check if an object is empty
export const isObjectEmpty = (obj) => {
  // Remove the id key from the object
  const objectWithoutId = { ...obj };
  delete objectWithoutId.id;
  return Object.values(objectWithoutId).every((x) => x === null || x === "" || typeof x === "undefined");
};

// Function to unnormalize text
export const unnormalize = (text) => {
  return text.replace(/-/g, " ");
};

export const addParamToQueryString = (param, value, state) => {
  const searchParams = new URLSearchParams(window.location.search);
  if (searchParams.has(param)) {
    searchParams.delete(param);
  }
  searchParams.set(param, value);
  modifyCurrentUrl(searchParams.toString(), state);
};

export const removeParamFromQueryString = (param, state) => {
  const searchParams = new URLSearchParams(window.location.search);
  if (searchParams.has(param)) {
    searchParams.delete(param);
    modifyCurrentUrl(searchParams.toString(), state);
  }
};

export const getParamFromQueryString = (param) => {
  const searchParams = new URLSearchParams(window.location.search);
  return searchParams.get(param);
};

export const modifyCurrentUrl = (params, state) => {
  const newUrl = `${window.location.pathname}${params ? `?${params}` : ""}`;
  history.pushState(state || {}, "new_url", newUrl);
};

// Detect if the job preview modal should be displayed
export const shouldDisplayJobModal = () => {
  // Check if the job_id parameter is present in the URL
  const job_id = getParamFromQueryString("job_id");
  if (job_id) {
    const elId = `job-preview-${job_id}`;
    // Check if the job preview button exists
    const jobPreviewBtn = document.getElementById(elId);
    if (jobPreviewBtn) {
      // Process the button
      htmx.process(jobPreviewBtn);
      // Open the modal
      htmx.trigger(jobPreviewBtn, "open-modal");
    }
  }
};
