/**
 * Returns the default SweetAlert2 options for consistent styling.
 * @returns {Object} SweetAlert2 options object for consistent alert appearance.
 */
const getAlertOptions = () => {
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
 * Shows a success alert with a custom message.
 * @param {string} message - The message to display in the alert.
 * @returns {void}
 */
export const showSuccessAlert = (message) => {
  Swal.fire({
    text: message,
    icon: "success",
    showConfirmButton: true,
    timer: 5000,
    ...getAlertOptions(),
  });
};

/**
 * Shows an error alert with a custom message.
 * @param {string} message - The message to display in the alert.
 * @returns {void}
 */
export const showErrorAlert = (message) => {
  Swal.fire({
    text: message,
    icon: "error",
    showConfirmButton: true,
    timer: 30000,
    ...getAlertOptions(),
  });
};

/**
 * Shows an informational alert with a custom message.
 * @param {string} message - The message to display in the alert.
 * @returns {void}
 */
export const showInfoAlert = (message) => {
  Swal.fire({
    text: message,
    icon: "info",
    showConfirmButton: true,
    timer: 10000,
    ...getAlertOptions(),
  });
};

/**
 * Shows an informational alert with custom HTML content.
 * @param {string} message - The HTML content to display in the alert.
 * @returns {void}
 */
export const showHtmlInfoAlert = (message) => {
  Swal.fire({
    html: message,
    icon: "info",
    showConfirmButton: true,
    timer: 10000,
    ...getAlertOptions(),
  });
};

/**
 * Shows a confirmation alert and triggers an event if confirmed.
 * @param {string} message - The message to display in the alert.
 * @param {string} btnId - The ID of the element to trigger if confirmed.
 * @param {string} confirmationMessage - The text for the confirmation button.
 * @returns {void}
 */
export const showConfirmationAlert = (message, btnId, confirmationMessage) => {
  Swal.fire({
    text: message,
    icon: "warning",
    showCancelButton: true,
    confirmButtonText: confirmationMessage,
    cancelButtonText: "No",
    ...getAlertOptions(),
  }).then((result) => {
    if (result.isConfirmed) {
      htmx.trigger(`#${btnId}`, "confirmed");
    }
  });
};
