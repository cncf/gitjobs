// Show or hide the provided modal.
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
export const shouldDisplayJobModal = (on_load = false) => {
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
      if (on_load) {
        // If the page is loaded, we need to trigger the modal
        // with the open-modal event (register view)
        htmx.trigger(jobPreviewBtn, "open-modal");
      } else {
        // If the page is not loaded, we need to trigger the modal
        // with the open-modal-on-popstate event (do not register view)
        htmx.trigger(jobPreviewBtn, "open-modal-on-popstate");
      }
    }
  }
};

export const registerJobIdView = async (job_id) => {
  try {
    fetch(`/jobs/${job_id}/views`, {
      method: "POST",
    });
  } catch (error) {
    // Do not do anything
  }
};

const NUMBER_REGEX = /\.0+$|(\.[0-9]*[1-9])0+$/;

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
