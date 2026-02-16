import "/static/vendor/js/sharer.v0.5.3.min.js";

/**
 * Sets up social media sharing links for a job posting.
 * Configures sharer metadata and fallback URLs for each platform.
 */
export const shareJob = () => {
  const socialLinks = document.getElementById("social-links");
  if (!socialLinks) {
    return;
  }

  const jobId = socialLinks.dataset.jobId;
  const shareUrl = `${window.location.origin}?job_id=${jobId}`;
  const subject = "Check out this job I found on GitJobs!";
  const message = "Check out this job I found on GitJobs!";
  const fallbackLinks = {
    twitter: `https://twitter.com/intent/tweet?text=${encodeURIComponent(message)}&url=${shareUrl}`,
    facebook: `https://www.facebook.com/sharer/sharer.php?u=${shareUrl}&quote=${encodeURIComponent(message)}`,
    linkedin: `https://www.linkedin.com/sharing/share-offsite/?url=${shareUrl}`,
    email: `mailto:?subject=${encodeURIComponent(subject)}&body=${encodeURIComponent(`${message} ${shareUrl}`)}`,
  };

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
  const copyLink = document.querySelector("#copy-link");
  if (copyLink && copyLink.dataset.copyInitialized !== "true") {
    copyLink.addEventListener("click", (event) => {
      event.preventDefault();
      navigator.clipboard.writeText(shareUrl);
      const tooltip = document.querySelector("#copy-link-tooltip");
      if (tooltip) {
        tooltip.classList.add("opacity-100", "z-10");
        setTimeout(() => {
          tooltip.classList.remove("opacity-100", "z-10");
        }, 3000);
      }
    });
    copyLink.dataset.copyInitialized = "true";
  }
};
