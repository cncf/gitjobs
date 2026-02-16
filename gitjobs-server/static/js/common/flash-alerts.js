import { showErrorAlert, showSuccessAlert } from "/static/js/common/alerts.js";

/**
 * Displays flash alerts rendered in the DOM.
 * @param {Document|HTMLElement} [root=document] - Root where flash alerts are searched
 */
export const displayFlashAlerts = (root = document) => {
  const selector = "[data-flash-alert]";
  const flashAlerts = [];

  if (root instanceof Element && root.matches(selector)) {
    flashAlerts.push(root);
  }

  if (typeof root.querySelectorAll === "function") {
    flashAlerts.push(...root.querySelectorAll(selector));
  }

  flashAlerts.forEach((flashAlert) => {
    if (flashAlert.dataset.flashAlertShown === "true") {
      return;
    }

    const level = flashAlert.dataset.flashAlert;
    const message = flashAlert.dataset.flashMessage;
    if (!message) {
      return;
    }

    if (level === "success") {
      showSuccessAlert(message);
    } else if (level === "error") {
      showErrorAlert(message);
    }

    flashAlert.dataset.flashAlertShown = "true";
  });
};

/**
 * Initializes flash alert rendering on initial load and HTMX lifecycle events.
 */
export const initializeFlashAlerts = () => {
  displayFlashAlerts();

  if (document.__gitjobsFlashAlertsBound) {
    return;
  }

  document.addEventListener("htmx:afterSwap", (event) => {
    displayFlashAlerts(event.target);
  });
  document.addEventListener("htmx:historyRestore", () => {
    displayFlashAlerts(document);
  });
  window.addEventListener("pageshow", () => {
    displayFlashAlerts(document);
  });

  document.__gitjobsFlashAlertsBound = "true";
};

initializeFlashAlerts();
