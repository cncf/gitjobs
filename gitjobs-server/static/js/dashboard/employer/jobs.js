import { processNewHtmxUrl } from "/static/js/common/common.js";

// Trigger action on the form provided.
export const triggerActionOnForm = (formId, action) => {
  const form = document.getElementById(formId);
  if (form) {
    htmx.trigger(form, action);
  }
};
