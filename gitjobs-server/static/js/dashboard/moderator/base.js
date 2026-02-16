import { initializeCloseMenuControls, initializeOpenMenuButton } from "/static/js/dashboard/base.js";

/**
 * Initializes moderator dashboard drawer controls.
 */
export const initializeModeratorBase = () => {
  initializeOpenMenuButton();
  initializeCloseMenuControls();
};

initializeModeratorBase();
