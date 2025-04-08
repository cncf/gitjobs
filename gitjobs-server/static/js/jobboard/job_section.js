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
      // Open external link in a new tab
      applyButton.addEventListener("click", () => {
        window.open(applyUrl, "_blank");
      });
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

export const renderEmbedCode = () => {
  const embedCode = document.getElementById("embed-code");
  const params = new URLSearchParams(window.location.search);
  params.append("limit", "10");
  embedCode.textContent = `
<iframe id="gitjobs" src="${window.location.origin}/embed?${params.toString()}" style="width:100%;max-width:870px;height:100%;display:block;border:none;"></iframe>

<!-- Uncomment the following lines for resizing iframes dynamically using open-iframe-resizer
<script type="module">
  import { initialize } from "https://cdn.jsdelivr.net/npm/@open-iframe-resizer/core@latest/dist/index.js";
  initialize({}, "#gitjobs");
</script> -->`;
};

export const copyEmbedCodeToClipboard = (elId) => {
  const embedCode = document.getElementById(elId);

  // Copy the text inside the text field
  navigator.clipboard.writeText(embedCode.textContent);

  showSuccessAlert("Embed code copied to clipboard!");
};
