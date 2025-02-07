import { processNewHtmxUrl } from "/static/js/common/common.js";

// Trigger action on the form provided.
export const triggerActionOnForm = (formId, action) => {
  const form = document.getElementById(formId);
  if (form) {
    htmx.trigger(form, action);
  }
};

export const updateJobsList = () => {
  const refreshButton = document.getElementById('refresh-table');
  htmx.trigger(refreshButton, 'click');
}

