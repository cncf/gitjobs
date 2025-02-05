// Show or hide the provided modal.
export const toggleModalVisibility = (modalId) => {
  const modal = document.getElementById(modalId);
  if (modal.classList.contains("hidden")) {
    modal.classList.remove("hidden");
    // This is used to hide body scroll when the modal is open
    modal.dataset.open = "true";
  } else {
    modal.classList.add("hidden");
    // This is used to show body scroll when the modal is open
    modal.dataset.open = "false";
  }
};

// Show or hide the provided drawer.
export const toggleDrawerVisibility = (drawerId) => {
  const drawer = document.getElementById(drawerId);
  if (drawer.classList.contains("-translate-x-full")) {
    drawer.classList.remove("-translate-x-full");
    // This is used to hide body scroll when the drawer is open
    drawer.dataset.open = "true";
  } else {
    drawer.classList.add("-translate-x-full");
    // This is used to show body scroll when the drawer is open
    drawer.dataset.open = "false";
  }
};

// Function to delay the execution of a function
export const debounce = (func, timeout = 300) => {
  let timer;
  return (...args) => {
    clearTimeout(timer);
    timer = setTimeout(() => { func.apply(this, args); }, timeout);
  };
}

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
