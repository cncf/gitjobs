import {
  handleHtmxResponse,
  showConfirmAlert,
  showInfoAlert,
  showSuccessAlert,
} from "/static/js/common/alerts.js";
import {
  lockBodyScroll,
  removeParamFromQueryString,
  toggleModalVisibility,
} from "/static/js/common/common.js";

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
          handleHtmxResponse({
            xhr: e.detail.xhr,
            successMessage: "You have successfully applied to this job!",
            errorMessage: "An error occurred applying to this job. Please try again later.",
          });
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
 * Initializes preview and embed modal interactions for job details.
 */
export const initializeJobPreviewModal = () => {
  const previewModal = document.getElementById("preview-modal");
  if (
    previewModal &&
    previewModal.dataset.open === "true" &&
    !previewModal.classList.contains("hidden") &&
    previewModal.dataset.initialScrollLockApplied !== "true"
  ) {
    lockBodyScroll();
    previewModal.dataset.initialScrollLockApplied = "true";
  }

  const onCloseModal = () => {
    const previewContent = document.getElementById("preview-content");
    if (previewContent) {
      previewContent.scrollTop = 0;
    }

    if (previewModal) {
      previewModal.dataset.initialScrollLockApplied = "false";
    }

    removeParamFromQueryString("job_id", {
      modal_preview: false,
    });
    toggleModalVisibility("preview-modal", "close");
  };

  const backdropPreviewModal = document.getElementById("backdrop-preview-modal");
  if (backdropPreviewModal && backdropPreviewModal.dataset.closeBound !== "true") {
    backdropPreviewModal.addEventListener("click", onCloseModal);
    backdropPreviewModal.dataset.closeBound = "true";
  }

  const closePreviewModal = document.getElementById("close-preview-modal");
  if (closePreviewModal && closePreviewModal.dataset.closeBound !== "true") {
    closePreviewModal.addEventListener("click", onCloseModal);
    closePreviewModal.dataset.closeBound = "true";
  }

  const tabs = document.querySelectorAll(".tab");
  tabs.forEach((tab) => {
    if (tab.dataset.tabBound === "true") {
      return;
    }

    tab.addEventListener("click", (event) => {
      const section = event.currentTarget.getAttribute("data-section");
      const buttons = document.querySelectorAll("#embed-code-modal [data-section]");
      buttons.forEach((button) => {
        button.setAttribute("data-active", "false");
        button.classList.remove("active");
      });
      event.currentTarget.setAttribute("data-active", "true");
      event.currentTarget.classList.add("active");

      const sections = document.querySelectorAll("#embed-code-modal .sections > div");
      sections.forEach((content) => {
        if (content.id !== section) {
          content.classList.add("hidden");
        } else {
          content.classList.remove("hidden");
        }
      });
    });

    tab.dataset.tabBound = "true";
  });

  const embedCodeButton = document.getElementById("embed-code-button");
  if (embedCodeButton && embedCodeButton.dataset.embedOpenBound !== "true") {
    embedCodeButton.addEventListener("click", () => {
      toggleModalVisibility("embed-code-modal", "open");
    });
    embedCodeButton.dataset.embedOpenBound = "true";
  }

  const closeEmbedCodeModal = document.getElementById("close-embed-code-modal");
  if (closeEmbedCodeModal && closeEmbedCodeModal.dataset.embedCloseBound !== "true") {
    closeEmbedCodeModal.addEventListener("click", () => {
      toggleModalVisibility("embed-code-modal", "close");
    });
    closeEmbedCodeModal.dataset.embedCloseBound = "true";
  }

  const embedCodeModalBackdrop = document.getElementById("backdrop-embed-code-modal");
  if (embedCodeModalBackdrop && embedCodeModalBackdrop.dataset.embedCloseBound !== "true") {
    embedCodeModalBackdrop.addEventListener("click", () => {
      toggleModalVisibility("embed-code-modal", "close");
    });
    embedCodeModalBackdrop.dataset.embedCloseBound = "true";
  }

  const copyButtons = document.querySelectorAll("[data-copy-button]");
  copyButtons.forEach((copyButton) => {
    if (copyButton.dataset.copyBound === "true") {
      return;
    }

    copyButton.addEventListener("click", () => {
      const content = copyButton.dataset.copyContent || "";
      navigator.clipboard.writeText(content);

      const tooltipId = copyButton.dataset.tooltipId;
      if (!tooltipId) {
        return;
      }

      const tooltip = document.getElementById(tooltipId);
      if (tooltip) {
        tooltip.classList.add("opacity-100", "z-10");
        setTimeout(() => {
          tooltip.classList.remove("opacity-100", "z-10");
        }, 3000);
      }
    });

    copyButton.dataset.copyBound = "true";
  });
};
