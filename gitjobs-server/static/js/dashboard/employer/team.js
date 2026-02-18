import { initializeConfirmHtmxButtons } from "/static/js/common/alerts.js";
import { initializeModalCloseHandlers, toggleModalVisibility } from "/static/js/common/common.js";

const ADD_MEMBER_BUTTON_ID = "add-member-button";
const ADD_MEMBER_MODAL_ID = "add-member-modal";
const CLOSE_ADD_MEMBER_MODAL_BUTTON_ID = "close-add-member-modal";
const BACKDROP_ADD_MEMBER_MODAL_ID = "backdrop-add-member-modal";

/**
 * Initializes employer team members page interactions.
 */
export const initializeEmployerTeamMembersList = () => {
  const addMemberButton = document.getElementById(ADD_MEMBER_BUTTON_ID);
  if (addMemberButton && addMemberButton.dataset.addMemberBound !== "true") {
    addMemberButton.addEventListener("click", () => {
      toggleModalVisibility(ADD_MEMBER_MODAL_ID, "open");
    });
    addMemberButton.dataset.addMemberBound = "true";
  }

  initializeModalCloseHandlers({
    modalId: ADD_MEMBER_MODAL_ID,
    triggerIds: [CLOSE_ADD_MEMBER_MODAL_BUTTON_ID, BACKDROP_ADD_MEMBER_MODAL_ID],
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
