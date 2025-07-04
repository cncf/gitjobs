import {
  showConfirmAlert,
  showErrorAlert,
  showInfoAlert,
  showSuccessAlert,
} from "/static/js/common/alerts.js";
import { isSuccessfulXHRStatus } from "/static/js/common/common.js";

/**
 * Initializes the job application button functionality.
 * Handles different states: logged out, external URL, no profile.
 */
export const initializeApplyButton = () => {
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
      showInfoAlert(
        "You need to be <a href='/log-in' class='underline font-medium' hx-boost='true'>logged in</a> to apply.",
        true,
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
          showInfoAlert(
            "You need to <a href='/dashboard/job-seeker' class='underline font-medium' hx-boost='true'>set up</a> your job seeker profile to apply.",
            true,
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

/**
 * Generates and displays the embed code for job listings.
 * Creates an iframe with current search parameters.
 */
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

/**
 * Copies embed code to clipboard and shows success message.
 * @param {string} elementId - ID of element containing embed code
 */
export const copyEmbedCodeToClipboard = (elementId) => {
  const embedCodeElement = document.getElementById(elementId);

  navigator.clipboard.writeText(embedCodeElement.textContent);

  showSuccessAlert("Embed code copied to clipboard!");
};

/**
 * Sets up social media sharing links for a job posting.
 * Configures share URLs for Twitter, Facebook, LinkedIn, and email.
 */
export const shareJob = () => {
  const socialLinks = document.getElementById("social-links");
  if (socialLinks) {
    const jobId = socialLinks.dataset.jobId;
    const shareUrl = `${window.location.origin}?job_id=${jobId}`;
    const subject = "Check out this job I found on GitJobs!";
    const message = "Check out this job I found on GitJobs!";

    const anchorTags = socialLinks.querySelectorAll("a");
    anchorTags.forEach((anchorTag) => {
      const platform = anchorTag.dataset.platform;

      switch (platform) {
        case "twitter":
          anchorTag.setAttribute(
            "href",
            `https://twitter.com/intent/tweet?text=${encodeURIComponent(message)}&url=${shareUrl}`,
          );
          break;
        case "facebook":
          anchorTag.setAttribute(
            "href",
            `https://www.facebook.com/sharer/sharer.php?u=${shareUrl}&quote=${encodeURIComponent(message)}`,
          );
          break;
        case "linkedin":
          anchorTag.setAttribute("href", `https://www.linkedin.com/sharing/share-offsite/?url=${shareUrl}`);
          break;
        case "email":
          anchorTag.setAttribute(
            "href",
            `mailto:?subject=${encodeURIComponent(subject)}&body=${encodeURIComponent(
              `${message} ${shareUrl}`,
            )}`,
          );
          break;
      }
    });

    // Copy link to clipboard
    const copyLink = document.querySelector("#copy-link");
    if (copyLink) {
      copyLink.addEventListener("click", () => {
        navigator.clipboard.writeText(shareUrl);
        const tooltip = document.querySelector("#copy-link-tooltip");
        if (tooltip) {
          tooltip.classList.add("opacity-100", "z-10");
          setTimeout(() => {
            tooltip.classList.remove("opacity-100", "z-10");
          }, 3000);
        }
      });
    }
  }
};
