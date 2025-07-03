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
 */
export const showErrorAlert = (message, withHtml = false) => {
  const alertOptions = {
    text: message,
    icon: "error",
    showConfirmButton: true,
    timer: 30000,
    ...getCommonAlertOptions(),
  };
  if (withHtml) {
    alertOptions.html = message; // Use HTML content if specified
  }

  Swal.fire(alertOptions);
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
