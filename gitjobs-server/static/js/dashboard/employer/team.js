import { initializeConfirmHtmxButtons } from "/static/js/common/alerts.js";
import { toggleModalVisibility } from "/static/js/common/common.js";

/**
 * Initializes employer team members page interactions.
 */
export const initializeEmployerTeamMembersList = () => {
  const addMemberButton = document.getElementById("add-member-button");
  if (addMemberButton && addMemberButton.dataset.addMemberBound !== "true") {
    addMemberButton.addEventListener("click", () => {
      toggleModalVisibility("add-member-modal", "open");
    });
    addMemberButton.dataset.addMemberBound = "true";
  }

  const backdropAddMemberModal = document.getElementById("backdrop-add-member-modal");
  if (backdropAddMemberModal && backdropAddMemberModal.dataset.backdropCloseBound !== "true") {
    backdropAddMemberModal.addEventListener("click", () => {
      toggleModalVisibility("add-member-modal", "close");
    });
    backdropAddMemberModal.dataset.backdropCloseBound = "true";
  }

  const closeAddMemberModal = document.getElementById("close-add-member-modal");
  if (closeAddMemberModal && closeAddMemberModal.dataset.closeButtonBound !== "true") {
    closeAddMemberModal.addEventListener("click", () => {
      toggleModalVisibility("add-member-modal", "close");
    });
    closeAddMemberModal.dataset.closeButtonBound = "true";
  }

  initializeConfirmHtmxButtons({
    selector: "[data-remove-member-button]",
    confirmMessage: "Are you sure you would like to delete this team member?",
    errorMessage: "Something went wrong deleting this team member. Please try again later.",
  });
};

/**
 * Initializes employer team invitations page interactions.
 */
export const initializeEmployerTeamInvitationsList = () => {
  initializeConfirmHtmxButtons({
    selector: "[data-reject-invitation-button]",
    confirmMessage: "Are you sure you would like to reject this invitation?",
    errorMessage: "Something went wrong rejecting this invitation. Please try again later.",
  });
};
