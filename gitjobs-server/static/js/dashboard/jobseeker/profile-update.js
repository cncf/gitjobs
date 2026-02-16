import { handleHtmxResponse, initializePreviewButtons } from "/static/js/common/alerts.js";
import { bindHtmxAfterRequestOnce, bindHtmxBeforeRequestOnce } from "/static/js/common/common.js";
import { displayActiveSection, validateFormData } from "/static/js/dashboard/jobseeker/form.js";
import { initializePendingChangesAlert } from "/static/js/dashboard/jobseeker/pending-changes-alert.js";

/**
 * Initializes the job seeker profile update page behaviors.
 */
export const initializeJobSeekerProfileUpdate = () => {
  const sectionsBtn = document.querySelectorAll("[data-section]");
  sectionsBtn.forEach((button) => {
    if (button.dataset.sectionSwitchBound === "true") {
      return;
    }

    button.addEventListener("click", () => {
      const section = button.getAttribute("data-section");
      displayActiveSection(section);
    });

    button.dataset.sectionSwitchBound = "true";
  });

  const pendingChangesAlert = initializePendingChangesAlert({
    alertId: "pending-changes-alert",
    formIds: ["profile-form", "experience-form", "education-form", "projects-form"],
  });

  bindHtmxBeforeRequestOnce({
    selector: "#update-profile-button",
    handler: (event) => {
      if (!validateFormData()) {
        event.preventDefault();
      }
    },
    boundAttribute: "profileUpdateBeforeRequestBound",
  });

  bindHtmxAfterRequestOnce({
    selector: "#update-profile-button",
    handler: (event) => {
      if (
        handleHtmxResponse({
          xhr: event.detail.xhr,
          successMessage: "Profile updated successfully.",
          errorMessage: "Something went wrong updating the profile. Please try again later.",
        })
      ) {
        pendingChangesAlert.markCurrentAsClean();
      }
    },
    boundAttribute: "profileUpdateAfterRequestBound",
  });

  initializePreviewButtons({
    selector: "#preview-button",
    invalidMessage: "You must fill in all required fields to be able to preview the profile.",
    errorMessage: "Something went wrong previewing the data. Please try again later.",
  });
};
