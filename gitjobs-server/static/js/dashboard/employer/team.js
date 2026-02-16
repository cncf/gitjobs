import { initializeConfirmHtmxButtons } from "/static/js/common/alerts.js";
import { initializeModalCloseHandlers, toggleModalVisibility } from "/static/js/common/common.js";

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

  initializeModalCloseHandlers({
    modalId: "add-member-modal",
    triggerIds: ["close-add-member-modal", "backdrop-add-member-modal"],
  });

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
