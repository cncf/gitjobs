import { handleHtmxResponse, initializePreviewButtons } from "/static/js/common/alerts.js";
import { toggleModalVisibility } from "/static/js/common/common.js";

/**
 * Initializes moderation actions for approve and reject job workflows.
 */
export const initializeModeratorJobs = () => {
  const approveButtons = document.querySelectorAll("[data-approve-job-button]");
  approveButtons.forEach((button) => {
    if (button.dataset.approveBound === "true") {
      return;
    }

    button.addEventListener("htmx:afterRequest", (event) => {
      handleHtmxResponse({
        xhr: event.detail.xhr,
        errorMessage: "Something went wrong approving this job. Please try again later.",
      });
    });

    button.dataset.approveBound = "true";
  });

  const rejectButtons = document.querySelectorAll(".reject-modal");
  rejectButtons.forEach((button) => {
    if (button.dataset.rejectOpenBound === "true") {
      return;
    }

    button.addEventListener("click", (event) => {
      const jobId = event.currentTarget.dataset.jobId;
      const rejectJobForm = document.getElementById("reject-job-form");
      if (!rejectJobForm || !jobId) {
        return;
      }

      rejectJobForm.setAttribute("hx-put", `/dashboard/moderator/jobs/${jobId}/reject`);
      htmx.process(rejectJobForm);
      toggleModalVisibility("reject-modal", "open");
    });

    button.dataset.rejectOpenBound = "true";
  });

  const rejectJobForm = document.getElementById("reject-job-form");
  if (rejectJobForm && rejectJobForm.dataset.rejectSubmitBound !== "true") {
    rejectJobForm.addEventListener("htmx:afterRequest", (event) => {
      if (
        handleHtmxResponse({
          xhr: event.detail.xhr,
          errorMessage: "Something went wrong rejecting this job. Please try again later.",
        })
      ) {
        rejectJobForm.reset();
        toggleModalVisibility("reject-modal", "close");
      }
    });

    rejectJobForm.dataset.rejectSubmitBound = "true";
  }

  const closeRejectModal = document.getElementById("close-reject-modal");
  if (closeRejectModal && closeRejectModal.dataset.rejectCloseBound !== "true") {
    closeRejectModal.addEventListener("click", () => {
      toggleModalVisibility("reject-modal", "close");
    });
    closeRejectModal.dataset.rejectCloseBound = "true";
  }

  const rejectModalBackdrop = document.getElementById("backdrop-reject-modal");
  if (rejectModalBackdrop && rejectModalBackdrop.dataset.rejectBackdropBound !== "true") {
    rejectModalBackdrop.addEventListener("click", () => {
      toggleModalVisibility("reject-modal", "close");
    });
    rejectModalBackdrop.dataset.rejectBackdropBound = "true";
  }
};

/**
 * Initializes moderator preview buttons.
 */
export const initializeModeratorPreviewButtons = () => {
  initializePreviewButtons({
    errorMessage: "Something went wrong previewing the data. Please try again later.",
  });
};
