import {
  showConfirmAlert,
  showErrorAlert,
  showInfoAlertWithHtml,
  showSuccessAlert,
} from "/static/js/common/alerts.js";
import { isSuccessfulXHRStatus } from "/static/js/common/common.js";

export const applyButton = () => {
  const applyButton = document.getElementById("apply-button");
  if (!applyButton) {
    return;
  }

  const applyUrl = applyButton.dataset.applyUrl;
  const userButton = document.getElementById("user-dropdown-button");
  const isUserLoggedIn = userButton.dataset.loggedIn;
  const hasProfile = userButton.dataset.hasProfile;
  applyButton.removeAttribute("disabled");

  if (isUserLoggedIn === "false") {
    applyButton.addEventListener("click", () => {
      showInfoAlertWithHtml(
        "You need to be <a href='/log-in' class='underline font-medium' hx-boost='true'>logged in</a> to apply.",
      );
    });
  } else {
    if (applyUrl !== "") {
      const applyAnchor = document.createElement("a");
      applyAnchor.href = applyUrl;
      applyAnchor.className = "btn-primary w-full block mt-4 mb-2";
      applyAnchor.textContent = "Apply";
      applyAnchor.target = "_blank";
      applyAnchor.rel = "noopener noreferrer";
      applyButton.replaceWith(applyAnchor);
    } else {
      if (hasProfile === "false") {
        applyButton.addEventListener("click", () => {
          showInfoAlertWithHtml(
            "You need to <a href='/dashboard/job-seeker' class='underline font-medium' hx-boost='true'>set up</a> your job seeker profile to apply.",
          );
        });
      } else {
        const jobId = applyButton.dataset.jobId;
        applyButton.setAttribute("hx-post", `/jobs/${jobId}/apply`);
        applyButton.setAttribute("hx-trigger", "confirmed");
        htmx.process(applyButton);
        applyButton.addEventListener("click", () => {
          showConfirmAlert("Are you sure you want to apply to this job?", "apply-button", "Yes");
        });

        applyButton.addEventListener("htmx:afterRequest", (e) => {
          if (isSuccessfulXHRStatus(e.detail.xhr.status)) {
            showSuccessAlert("You have successfully applied to this job!");
          } else {
            showErrorAlert("An error occurred applying to this job, please try again later.");
          }
        });
      }
    }
  }
};
