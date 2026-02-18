import { handleHtmxResponse, initializePreviewButtons } from "/static/js/common/alerts.js";
import {
  bindHtmxAfterRequestOnce,
  initializeModalCloseHandlers,
  toggleModalVisibility,
} from "/static/js/common/common.js";

const REJECT_MODAL_ID = "reject-modal";
const REJECT_JOB_FORM_ID = "reject-job-form";
const REJECT_JOB_FORM_SELECTOR = `#${REJECT_JOB_FORM_ID}`;
const CLOSE_REJECT_MODAL_BUTTON_ID = "close-reject-modal";
const BACKDROP_REJECT_MODAL_ID = "backdrop-reject-modal";

/**
 * Initializes moderation actions for approve and reject job workflows.
 */
export const initializeModeratorJobs = () => {
  bindHtmxAfterRequestOnce({
    selector: "[data-approve-job-button]",
    handler: (event) => {
      handleHtmxResponse({
        xhr: event.detail.xhr,
        errorMessage: "Something went wrong approving this job. Please try again later.",
      });
    },
    boundAttribute: "approveBound",
  });

  const rejectButtons = document.querySelectorAll(".reject-modal");
  rejectButtons.forEach((button) => {
    if (button.dataset.rejectOpenBound === "true") {
      return;
    }

    button.addEventListener("click", (event) => {
      const jobId = event.currentTarget.dataset.jobId;
      const rejectJobForm = document.getElementById(REJECT_JOB_FORM_ID);
      if (!rejectJobForm || !jobId) {
        return;
      }

      rejectJobForm.setAttribute("hx-put", `/dashboard/moderator/jobs/${jobId}/reject`);
      const htmxInstance = window.htmx;
      if (typeof htmxInstance?.process === "function") {
        htmxInstance.process(rejectJobForm);
      }
      toggleModalVisibility(REJECT_MODAL_ID, "open");
    });

    button.dataset.rejectOpenBound = "true";
  });

  bindHtmxAfterRequestOnce({
    selector: REJECT_JOB_FORM_SELECTOR,
    handler: (event) => {
      if (
        handleHtmxResponse({
          xhr: event.detail.xhr,
          errorMessage: "Something went wrong rejecting this job. Please try again later.",
        })
      ) {
        const rejectJobForm = event.currentTarget;
        if (!(rejectJobForm instanceof HTMLFormElement)) {
          return;
        }
        rejectJobForm.reset();
        toggleModalVisibility(REJECT_MODAL_ID, "close");
      }
    },
    boundAttribute: "rejectSubmitBound",
  });

  initializeModalCloseHandlers({
    modalId: REJECT_MODAL_ID,
    triggerIds: [CLOSE_REJECT_MODAL_BUTTON_ID, BACKDROP_REJECT_MODAL_ID],
  });
};

/**
 * Initializes moderator preview buttons.
 */
export const initializeModeratorPreviewButtons = () => {
  initializePreviewButtons({
    errorMessage: "Something went wrong previewing the data. Please try again later.",
  });
};
