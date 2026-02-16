import { handleHtmxResponse, initializePreviewButtons } from "/static/js/common/alerts.js";
import {
  bindHtmxAfterRequestOnce,
  initializeModalCloseHandlers,
  toggleModalVisibility,
} from "/static/js/common/common.js";

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

  bindHtmxAfterRequestOnce({
    selector: "#reject-job-form",
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
        toggleModalVisibility("reject-modal", "close");
      }
    },
    boundAttribute: "rejectSubmitBound",
  });

  initializeModalCloseHandlers({
    modalId: "reject-modal",
    triggerIds: ["close-reject-modal", "backdrop-reject-modal"],
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
