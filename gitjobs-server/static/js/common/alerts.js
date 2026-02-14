/**
 * Returns common configuration options for all alert dialogs.
 * Includes positioning, styling, and custom CSS classes.
 * @returns {Object} Alert configuration options for SweetAlert2
 */
const getCommonAlertOptions = () => {
  return {
    position: "top-end",
    buttonsStyling: false,
    iconColor: "var(--color-primary-500)",
    backdrop: false,
    customClass: {
      popup: "pb-10! pt-5! px-0! rounded-lg! max-w-[100%] md:max-w-[400px]! shadow-lg!",
      title: "text-md",
      htmlContainer: "text-base/6!",
      icon: "text-[0.4rem]! md:text-[0.5rem]!",
      confirmButton: "btn-primary",
      denyButton: "btn-primary-outline ms-5",
      cancelButton: "btn-primary-outline ms-5",
    },
  };
};

/**
 * Displays a success alert with the given message.
 * Auto-dismisses after 5 seconds.
 * @param {string} message - The success message to display
 */
export const showSuccessAlert = (message) => {
  Swal.fire({
    text: message,
    icon: "success",
    showConfirmButton: true,
    timer: 5000,
    ...getCommonAlertOptions(),
  });
};

/**
 * Displays an error alert with the given message.
 * Auto-dismisses after 30 seconds to ensure user sees errors.
 * @param {string} message - The error message to display
 * @param {boolean} withHtml - Whether to display the message as HTML content
 * @param {boolean} persist - Whether to keep the alert until dismissed
 */
export const showErrorAlert = (message, withHtml = false, persist = false) => {
  const alertOptions = {
    text: message,
    icon: "error",
    showConfirmButton: true,
    ...getCommonAlertOptions(),
  };
  if (!persist) {
    alertOptions.timer = 30000;
  }
  if (withHtml) {
    alertOptions.html = message; // Use HTML content if specified
  }

  Swal.fire(alertOptions);
};

/**
 * Displays a server error with a warning box when available (e.g., 422 errors).
 * @param {string} baseMessage - Fallback human message
 * @param {string} serverError - Raw server response text (optional)
 */
export const showServerErrorAlert = (baseMessage, serverError) => {
  const warningBox = serverError
    ? `<div class="mt-4 mb-2 rounded-lg border border-amber-200 bg-amber-50 px-4 py-3 text-sm text-amber-900 text-left">${serverError}</div>`
    : "";
  showErrorAlert(`${baseMessage}${warningBox}`, true, true);
};

/**
 * Removes retry guidance suffixes from error messages.
 * @param {string} message - Message to normalize
 * @returns {string} Normalized message
 */
const stripRetryMessage = (message) => {
  if (!message) {
    return message;
  }

  return message.replace(/\s*Please try again later\.?/i, "").trim();
};

/**
 * Handles common HTMX response patterns and displays alerts.
 * Returns true on success (2xx), false otherwise.
 * @param {Object} params - Response handling params
 * @param {XMLHttpRequest} params.xhr - XHR object from HTMX event
 * @param {string} params.successMessage - Optional success alert message
 * @param {string} params.errorMessage - Generic error alert message
 * @param {boolean} [params.errorWithHtml=false] - Render error message as HTML
 * @param {boolean} [params.treatUnprocessableAsGenericError=false] - Skip 422 server details
 * @returns {boolean} True on success status
 */
export const handleHtmxResponse = ({
  xhr,
  successMessage,
  errorMessage,
  errorWithHtml = false,
  treatUnprocessableAsGenericError = false,
}) => {
  if (!xhr) {
    showErrorAlert(errorMessage, errorWithHtml);
    return false;
  }

  if (xhr.status >= 200 && xhr.status < 300) {
    if (successMessage) {
      showSuccessAlert(successMessage);
    }
    return true;
  }

  if (xhr.status === 422 && !treatUnprocessableAsGenericError) {
    const cleanedErrorMessage = stripRetryMessage(errorMessage);
    showServerErrorAlert(cleanedErrorMessage, xhr.responseText?.trim());
    return false;
  }

  showErrorAlert(errorMessage, errorWithHtml);
  return false;
};

/**
 * Displays an informational alert with plain text message.
 * Auto-dismisses after 10 seconds.
 * @param {string} message - The info message to display
 * @param {boolean} withHtml - Whether to display the message as HTML content
 */
export const showInfoAlert = (message, withHtml = false) => {
  const alertOptions = {
    text: message,
    icon: "info",
    showConfirmButton: true,
    timer: 10000,
    ...getCommonAlertOptions(),
  };
  if (withHtml) {
    alertOptions.html = message; // Use HTML content if specified
  }
  Swal.fire(alertOptions);
};

/**
 * Displays a confirmation dialog with Yes/No options.
 * Triggers an HTMX 'confirmed' event on the specified button if confirmed.
 * @param {string} message - The confirmation message to display
 * @param {string} buttonId - ID of the button to trigger on confirmation
 * @param {string} confirmText - Text for the confirm button
 */
export const showConfirmAlert = (message, buttonId, confirmText) => {
  Swal.fire({
    text: message,
    icon: "warning",
    showCancelButton: true,
    confirmButtonText: confirmText,
    cancelButtonText: "No",
    ...getCommonAlertOptions(),
  }).then((result) => {
    if (result.isConfirmed) {
      htmx.trigger(`#${buttonId}`, "confirmed");
    }
  });
};
