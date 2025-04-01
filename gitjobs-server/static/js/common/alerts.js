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

export const showSuccessAlert = (message) => {
  Swal.fire({
    text: message,
    icon: "success",
    showConfirmButton: false,
    timer: 5000,
    ...getCommonAlertOptions(),
  });
};

export const showErrorAlert = (message) => {
  Swal.fire({
    text: message,
    icon: "error",
    showConfirmButton: true,
    timer: 30000,
    ...getCommonAlertOptions(),
  });
};

export const showInfoAlert = (message) => {
  Swal.fire({
    text: message,
    icon: "info",
    showConfirmButton: true,
    timer: 10000,
    ...getCommonAlertOptions(),
  });
};

export const showInfoAlertWithHtml = (message) => {
  Swal.fire({
    html: message,
    icon: "info",
    showConfirmButton: true,
    timer: 10000,
    ...getCommonAlertOptions(),
  });
};

export const showConfirmAlert = (message, btnId, confirmText) => {
  Swal.fire({
    text: message,
    icon: "warning",
    showCancelButton: true,
    confirmButtonText: confirmText,
    cancelButtonText: "No",
    ...getCommonAlertOptions(),
  }).then((result) => {
    if (result.isConfirmed) {
      htmx.trigger(`#${btnId}`, "confirmed");
    }
  });
};
