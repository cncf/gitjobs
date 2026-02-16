import { showConfirmAlert } from "/static/js/common/alerts.js";
import { ensureElementId } from "/static/js/common/common.js";

/**
 * Serializes a form field value for deterministic snapshot comparisons.
 * @param {FormDataEntryValue} value - Field value from FormData
 * @returns {string} Serialized value
 */
const serializeFormDataValue = (value) => {
  if (value instanceof File) {
    return `file:${value.name}:${value.size}:${value.lastModified}`;
  }
  return String(value ?? "");
};

/**
 * Builds a stable snapshot for all tracked forms.
 * @param {HTMLFormElement[]} forms - Tracked forms
 * @returns {string} URL-like snapshot payload
 */
const buildFormsSnapshot = (forms) => {
  const entries = [];
  forms.forEach((form) => {
    const formData = new FormData(form);
    for (const [name, value] of formData.entries()) {
      entries.push([`${form.id}:${name}`, serializeFormDataValue(value)]);
    }
  });

  entries.sort(([leftKey, leftValue], [rightKey, rightValue]) => {
    if (leftKey === rightKey) {
      return leftValue.localeCompare(rightValue);
    }
    return leftKey.localeCompare(rightKey);
  });

  return entries.map(([name, value]) => `${encodeURIComponent(name)}=${encodeURIComponent(value)}`).join("&");
};

/**
 * Sets up pending changes visibility and optional cancel confirmation.
 * @param {Object} config - Init configuration
 * @param {string} config.alertId - Alert element id
 * @param {string[]} config.formIds - Form ids included in the pending snapshot
 * @param {string} [config.cancelButtonId] - Cancel button id to guard
 * @param {string} [config.confirmMessage] - Confirmation text shown on cancel
 * @param {string} [config.confirmText] - Confirmation button label
 * @returns {{
 *   hasPendingChanges: () => boolean,
 *   refresh: () => void,
 *   markCurrentAsClean: () => void
 * }} API.
 */
export const initializePendingChangesAlert = ({
  alertId,
  formIds,
  cancelButtonId = "",
  confirmMessage = "",
  confirmText = "Leave",
}) => {
  const pendingChangesAlert = document.getElementById(alertId);
  if (pendingChangesAlert?.__pendingChangesAlertApi) {
    return pendingChangesAlert.__pendingChangesAlertApi;
  }
  const trackedForms = (formIds || []).map((formId) => document.getElementById(formId)).filter(Boolean);
  const cancelButton = cancelButtonId ? document.getElementById(cancelButtonId) : null;

  let initialFormSnapshot = "";
  let hasPendingChanges = false;
  let pendingChangesReady = false;
  let pendingChangesAnimationFrame = null;

  const setPendingChangesState = (dirty) => {
    hasPendingChanges = dirty;
    if (pendingChangesAlert) {
      pendingChangesAlert.classList.toggle("hidden", !dirty);
    }
  };

  const refreshPendingChangesState = () => {
    if (!pendingChangesReady) {
      return;
    }
    const currentSnapshot = buildFormsSnapshot(trackedForms);
    setPendingChangesState(currentSnapshot !== initialFormSnapshot);
  };

  const schedulePendingChangesRefresh = () => {
    if (pendingChangesAnimationFrame) {
      return;
    }
    pendingChangesAnimationFrame = requestAnimationFrame(() => {
      pendingChangesAnimationFrame = null;
      refreshPendingChangesState();
    });
  };

  const markCurrentAsClean = () => {
    if (!pendingChangesReady) {
      return;
    }
    initialFormSnapshot = buildFormsSnapshot(trackedForms);
    setPendingChangesState(false);
  };

  if (pendingChangesAlert && trackedForms.length > 0) {
    const handleTrackedEvent = (event) => {
      if (event.target instanceof Node) {
        schedulePendingChangesRefresh();
      }
    };

    trackedForms.forEach((form) => {
      form.addEventListener("input", handleTrackedEvent, true);
      form.addEventListener("change", handleTrackedEvent, true);
      form.addEventListener("click", handleTrackedEvent, true);
    });

    const observer = new MutationObserver((mutations) => {
      const shouldRefresh = mutations.some((mutation) =>
        trackedForms.some((form) => form.contains(mutation.target)),
      );
      if (shouldRefresh) {
        schedulePendingChangesRefresh();
      }
    });

    trackedForms.forEach((form) => {
      observer.observe(form, {
        childList: true,
        subtree: true,
        attributes: true,
        characterData: true,
      });
    });

    requestAnimationFrame(() => {
      initialFormSnapshot = buildFormsSnapshot(trackedForms);
      pendingChangesReady = true;
      refreshPendingChangesState();
    });
  }

  if (cancelButton && confirmMessage) {
    ensureElementId({
      element: cancelButton,
      prefix: "gitjobs-pending-changes-cancel-button",
      counterKey: "__gitjobsPendingChangesCancelButtonCounter",
    });

    if (cancelButton.dataset.pendingChangesCancelBound !== "true") {
      cancelButton.addEventListener("click", (event) => {
        if (!hasPendingChanges) {
          return;
        }
        event.preventDefault();
        event.stopImmediatePropagation();
        showConfirmAlert(confirmMessage, cancelButton.id, confirmText);
      });
      cancelButton.dataset.pendingChangesCancelBound = "true";
    }
  }

  const api = {
    hasPendingChanges: () => hasPendingChanges,
    refresh: refreshPendingChangesState,
    markCurrentAsClean,
  };

  if (pendingChangesAlert) {
    pendingChangesAlert.__pendingChangesAlertApi = api;
  }

  return api;
};
