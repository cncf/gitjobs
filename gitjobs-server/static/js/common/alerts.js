const getCommonAlertOptions = () => {
  return {
    position: "top-end",
    buttonsStyling: false,
    iconColor: "var(--primary-color)",
    backdrop: false,
    customClass: {
      popup: "pb-10 pt-5 px-0 rounded-2xl max-w-[400px] shadow-lg",
      title: "text-md",
      htmlContainer: "text-normal leading-6",
      icon: "text-[0.5rem]",
      confirmButton: "btn-primary",
      denyButton: "btn-primary-outline",
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
