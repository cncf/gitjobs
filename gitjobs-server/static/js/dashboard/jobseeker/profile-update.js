import { handleHtmxResponse, initializePreviewButtons } from "/static/js/common/alerts.js";
import { bindHtmxAfterRequestOnce, bindHtmxBeforeRequestOnce } from "/static/js/common/common.js";
import { displayActiveSection, validateFormData } from "/static/js/dashboard/jobseeker/form.js";
import { initializePendingChangesAlert } from "/static/js/dashboard/jobseeker/pending-changes-alert.js";

const UPDATE_PROFILE_BUTTON_SELECTOR = "#update-profile-button";
const PREVIEW_BUTTON_SELECTOR = "#preview-button";
const PENDING_CHANGES_ALERT_ID = "pending-changes-alert";
const PROFILE_FORM_ID = "profile-form";
const EXPERIENCE_FORM_ID = "experience-form";
const EDUCATION_FORM_ID = "education-form";
const PROJECTS_FORM_ID = "projects-form";

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
    alertId: PENDING_CHANGES_ALERT_ID,
    formIds: [PROFILE_FORM_ID, EXPERIENCE_FORM_ID, EDUCATION_FORM_ID, PROJECTS_FORM_ID],
  });

  bindHtmxBeforeRequestOnce({
    selector: UPDATE_PROFILE_BUTTON_SELECTOR,
    handler: (event) => {
      if (!validateFormData()) {
        event.preventDefault();
      }
    },
    boundAttribute: "profileUpdateBeforeRequestBound",
  });

  bindHtmxAfterRequestOnce({
    selector: UPDATE_PROFILE_BUTTON_SELECTOR,
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
    selector: PREVIEW_BUTTON_SELECTOR,
    invalidMessage: "You must fill in all required fields to be able to preview the profile.",
    errorMessage: "Something went wrong previewing the data. Please try again later.",
  });
};
