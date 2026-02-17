import { initializeCloseMenuControls, initializeOpenMenuButton } from "/static/js/dashboard/base.js";

/**
 * Initializes moderator dashboard drawer controls.
 */
const initializeModeratorBase = () => {
  initializeOpenMenuButton();
  initializeCloseMenuControls();
};

initializeModeratorBase();
