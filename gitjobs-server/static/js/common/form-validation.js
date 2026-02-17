/**
 * Form validation module for enforcing trimmed values and password confirmation.
 * Auto-wires all forms on the page.
 * @module form-validation
 */

import { passwordsMatch, trimmedNonEmpty } from "/static/js/common/validators.js";
import { isDashboardPath, isElementInView } from "/static/js/common/common.js";

const FIELD_SELECTOR =
  'input:not([type="hidden"]):not([type="file"]):not([type="checkbox"]):not([type="radio"]), textarea';

/**
 * Checks if a field is a password input.
 * @param {HTMLElement} field - The form field element
 * @returns {boolean} True if field is a password input
 */
const isPasswordField = (field) => field instanceof HTMLInputElement && field.type === "password";

/**
 * Normalizes a non-required field by trimming whitespace.
 * Skips password fields to preserve intentional spaces.
 * @param {HTMLInputElement|HTMLTextAreaElement} field - The form field
 */
const normalizeField = (field) => {
  if (isPasswordField(field)) {
    return;
  }

  const trimmedValue = field.value.trim();
  if (trimmedValue !== field.value) {
    field.value = trimmedValue;
  }
};

/**
 * Finds the nearest visible element for scrolling.
 * @param {HTMLElement} field - The invalid form field
 * @returns {HTMLElement|null} Visible element to scroll into view
 */
const getVisibleScrollTarget = (field) => {
  if (!field) {
    return null;
  }

  if (field.getClientRects().length > 0) {
    return field;
  }

  let currentElement = field.parentElement;
  while (currentElement) {
    if (currentElement.getClientRects().length > 0) {
      return currentElement;
    }
    currentElement = currentElement.parentElement;
  }

  return null;
};

/**
 * Returns header height when it is sticky or fixed.
 * @returns {number} Header height in pixels
 */
const getStickyHeaderOffset = () => {
  const headerNavigation = document.querySelector("nav[aria-label='Main navigation']");
  if (!headerNavigation) {
    return 0;
  }

  const style = window.getComputedStyle(headerNavigation);
  if (style.position !== "fixed" && style.position !== "sticky") {
    return 0;
  }

  const rect = headerNavigation.getBoundingClientRect();
  return rect.height || 0;
};

/**
 * Adjusts scroll position so target is not hidden behind a fixed header.
 * @param {HTMLElement} element - Element to adjust scroll for
 */
const adjustScrollForHeader = (element) => {
  if (!element || typeof element.getBoundingClientRect !== "function") {
    return;
  }

  const headerOffset = getStickyHeaderOffset();
  if (!headerOffset || typeof window.scrollBy !== "function") {
    return;
  }

  const gap = 50;
  const rect = element.getBoundingClientRect();
  if (rect.top < headerOffset + gap) {
    window.scrollBy({
      top: rect.top - headerOffset - gap,
      left: 0,
      behavior: "auto",
    });
  }
};

/**
 * Scrolls invalid fields into view on dashboard pages.
 * @param {HTMLElement} field - Invalid field
 */
const scrollToInvalidField = (field) => {
  if (!isDashboardPath()) {
    return;
  }

  const targetElement = getVisibleScrollTarget(field);
  if (!targetElement || typeof targetElement.scrollIntoView !== "function") {
    return;
  }

  if (!isElementInView(targetElement)) {
    targetElement.scrollIntoView({ behavior: "auto", block: "start" });
  }

  adjustScrollForHeader(targetElement);
};

let invalidScrollPending = false;

/**
 * Handles invalid events and scrolls to the first invalid field.
 * @param {Event} event - Invalid event fired by the browser
 */
const handleInvalidEvent = (event) => {
  if (invalidScrollPending) {
    return;
  }

  const field = event.target;
  if (!(field instanceof HTMLElement)) {
    return;
  }

  invalidScrollPending = true;

  const runScroll = () => {
    scrollToInvalidField(field);
    invalidScrollPending = false;
  };

  if (typeof requestAnimationFrame === "function") {
    requestAnimationFrame(() => setTimeout(runScroll, 0));
  } else {
    setTimeout(runScroll, 0);
  }
};

/**
 * Validates required fields and trims non-password values.
 * @param {HTMLInputElement|HTMLTextAreaElement} field - Field to validate
 * @returns {boolean} True if valid
 */
const validateRequiredField = (field) => {
  field.setCustomValidity("");

  const emptyError = trimmedNonEmpty(field.value);
  if (emptyError) {
    field.setCustomValidity(emptyError);
    field.reportValidity();
    return false;
  }

  if (!isPasswordField(field)) {
    normalizeField(field);
  }

  if (!field.checkValidity()) {
    field.reportValidity();
    return false;
  }

  return true;
};

/**
 * Validates password and confirmation fields match.
 * Uses data-password and data-password-confirmation attributes.
 * @param {HTMLFormElement} form - Form element
 * @returns {boolean} True if valid
 */
const validatePasswordConfirmation = (form) => {
  const passwordInput = form.querySelector("[data-password]");
  const confirmationInput = form.querySelector("[data-password-confirmation]");

  if (!passwordInput || !confirmationInput) {
    return true;
  }

  const error = passwordsMatch(passwordInput.value, confirmationInput.value);
  if (error) {
    confirmationInput.setCustomValidity(error);
    confirmationInput.reportValidity();
    return false;
  }

  confirmationInput.setCustomValidity("");
  return true;
};

/**
 * Validates all supported fields in a form.
 * @param {HTMLFormElement} form - Form to validate
 * @returns {boolean} True if all fields are valid
 */
const validateForm = (form) => {
  const fields = form.querySelectorAll(FIELD_SELECTOR);

  for (const field of fields) {
    if (field.disabled) {
      continue;
    }

    if (!field.required) {
      normalizeField(field);
      continue;
    }

    if (!validateRequiredField(field)) {
      return false;
    }
  }

  return validatePasswordConfirmation(form);
};

/**
 * Keeps password confirmation validity in sync while typing.
 * @param {HTMLFormElement} form - Form element
 */
const wirePasswordInputs = (form) => {
  const passwordInput = form.querySelector("[data-password]");
  const confirmationInput = form.querySelector("[data-password-confirmation]");
  if (!passwordInput || !confirmationInput) {
    return;
  }

  const syncValidity = () => {
    if (!passwordInput.value || !confirmationInput.value) {
      confirmationInput.setCustomValidity("");
      return;
    }

    const error = passwordsMatch(passwordInput.value, confirmationInput.value);
    confirmationInput.setCustomValidity(error ?? "");
  };

  passwordInput.addEventListener("input", syncValidity);
  confirmationInput.addEventListener("input", syncValidity);
};

/**
 * Clears custom validity on input for required fields.
 * @param {HTMLFormElement} form - Form element
 */
const wireRequiredInputs = (form) => {
  const fields = form.querySelectorAll(FIELD_SELECTOR);
  fields.forEach((field) => {
    if (!field.required) {
      return;
    }

    field.addEventListener("input", () => {
      field.setCustomValidity("");
    });
  });
};

/**
 * Validates forms included via hx-include attribute.
 * @param {HTMLElement} element - Element with hx-include
 * @returns {boolean} True if all included forms are valid
 */
const validateIncludedForms = (element) => {
  const includeAttr = element.getAttribute("hx-include");
  if (!includeAttr) {
    return true;
  }

  const selectors = includeAttr
    .split(",")
    .map((selector) => selector.trim())
    .filter(Boolean);

  for (const selector of selectors) {
    const includedElement = document.querySelector(selector);
    if (includedElement?.matches("form") && !validateForm(includedElement)) {
      return false;
    }
  }

  return true;
};

/**
 * Wires validation event listeners to a form.
 * Prevents double wiring with data-trimmed-ready attribute.
 * @param {HTMLFormElement} form - Form element
 */
const wireForm = (form) => {
  if (form.dataset.trimmedReady === "true") {
    return;
  }
  form.dataset.trimmedReady = "true";

  wirePasswordInputs(form);
  wireRequiredInputs(form);

  form.addEventListener("submit", (event) => {
    if (!validateForm(form)) {
      event.preventDefault();
      event.stopPropagation();
    }
  });

  form.addEventListener("htmx:configRequest", (event) => {
    const requestElement = event.detail?.elt;
    if (requestElement?.id === "cancel-button" || requestElement?.dataset?.skipValidation === "true") {
      return;
    }

    if (!validateForm(form)) {
      event.preventDefault();
    }
  });
};

/**
 * Handles htmx:configRequest events for form and hx-include validation.
 * @param {Event} event - HTMX config request event
 */
const handleConfigRequest = (event) => {
  const requestElement = event.detail?.elt || event.target;
  if (!(requestElement instanceof HTMLElement)) {
    return;
  }

  if (requestElement.id === "cancel-button" || requestElement.dataset?.skipValidation === "true") {
    return;
  }

  if (requestElement.matches("form")) {
    if (!validateForm(requestElement)) {
      event.preventDefault();
    }
    return;
  }

  if (!validateIncludedForms(requestElement)) {
    event.preventDefault();
  }
};

/**
 * Initializes validation for all forms in the page.
 */
const init = () => {
  document.querySelectorAll("form").forEach(wireForm);

  const htmxInstance = window.htmx;
  if (typeof htmxInstance?.onLoad === "function") {
    htmxInstance.onLoad((element) => {
      if (!element) {
        return;
      }

      if (element instanceof HTMLFormElement) {
        wireForm(element);
      }
      element.querySelectorAll?.("form").forEach(wireForm);
    });
  }

  document.body?.addEventListener("htmx:configRequest", handleConfigRequest);
  document.addEventListener("invalid", handleInvalidEvent, true);
};

if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", init);
} else {
  init();
}
