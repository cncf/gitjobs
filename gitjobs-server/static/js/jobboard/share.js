import "/static/vendor/js/sharer.v0.5.3.min.js";

/**
 * Builds share metadata and fallback links for a job ID.
 * @param {string} jobId - Job identifier used in the share URL
 * @returns {{message: string, subject: string, shareUrl: string, fallbackLinks: Object}}
 */
const getShareMetadata = (jobId) => {
  const shareUrl = `${window.location.origin}?job_id=${jobId}`;
  const encodedShareUrl = encodeURIComponent(shareUrl);
  const subject = "Check out this job I found on GitJobs!";
  const message = "Check out this job I found on GitJobs!";
  return {
    message,
    subject,
    shareUrl,
    fallbackLinks: {
      twitter: `https://twitter.com/intent/tweet?text=${encodeURIComponent(message)}&url=${encodedShareUrl}`,
      facebook: `https://www.facebook.com/sharer/sharer.php?u=${encodedShareUrl}&quote=${encodeURIComponent(message)}`,
      linkedin: `https://www.linkedin.com/sharing/share-offsite/?url=${encodedShareUrl}`,
      email: `mailto:?subject=${encodeURIComponent(subject)}&body=${encodeURIComponent(`${message} ${shareUrl}`)}`,
    },
  };
};

/**
 * Copies text to clipboard using Clipboard API with a fallback.
 * @param {string} content - Text content to copy
 * @returns {Promise<void>} Resolves when content is copied
 */
const copyToClipboard = async (content) => {
  if (navigator.clipboard?.writeText) {
    await navigator.clipboard.writeText(content);
    return;
  }

  const temporaryInput = document.createElement("textarea");
  temporaryInput.value = content;
  temporaryInput.setAttribute("readonly", "");
  temporaryInput.style.position = "absolute";
  temporaryInput.style.left = "-9999px";
  document.body.appendChild(temporaryInput);
  temporaryInput.select();
  document.execCommand("copy");
  document.body.removeChild(temporaryInput);
};

/**
 * Sets up social media sharing links for a job posting.
 * Configures sharer metadata and fallback URLs for each platform.
 * @param {Document|HTMLElement} root - Root element containing share links
 */
export const shareJob = (root = document) => {
  const socialLinksElements = root.querySelectorAll("#social-links");
  if (socialLinksElements.length === 0) {
    return;
  }

  socialLinksElements.forEach((socialLinks) => {
    const jobId = socialLinks.dataset.jobId;
    if (!jobId) {
      return;
    }

    const { message, subject, shareUrl, fallbackLinks } = getShareMetadata(jobId);

    const anchorTags = socialLinks.querySelectorAll("a[data-platform]");
    anchorTags.forEach((anchorTag) => {
      const platform = anchorTag.dataset.platform;
      const fallbackLink = fallbackLinks[platform];
      if (!fallbackLink) {
        return;
      }

      anchorTag.setAttribute("href", fallbackLink);
      anchorTag.setAttribute("data-sharer", platform);
      anchorTag.setAttribute("data-title", message);
      anchorTag.setAttribute("data-url", shareUrl);
      if (platform === "email") {
        anchorTag.setAttribute("data-subject", subject);
      }

      if (anchorTag.dataset.sharerInitialized === "true" || !window.Sharer) {
        return;
      }

      anchorTag.addEventListener("click", (event) => {
        event.preventDefault();
        const sharerInstance = new window.Sharer(anchorTag);
        sharerInstance.share();
      });
      anchorTag.dataset.sharerInitialized = "true";
    });

    // Copy link to clipboard
    const copyLink = socialLinks.querySelector("#copy-link");
    if (copyLink && copyLink.dataset.copyInitialized !== "true") {
      copyLink.addEventListener("click", async (event) => {
        event.preventDefault();
        await copyToClipboard(shareUrl);
        const tooltip = socialLinks.querySelector("#copy-link-tooltip");
        if (tooltip) {
          tooltip.classList.add("opacity-100", "z-10");
          setTimeout(() => {
            tooltip.classList.remove("opacity-100", "z-10");
          }, 3000);
        }
      });
      copyLink.dataset.copyInitialized = "true";
    }
  });
};
