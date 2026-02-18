import { initializeModalCloseHandlers, setDrawerVisibility } from "/static/js/common/common.js";

const DRAWER_MENU_ID = "drawer-menu";
const OPEN_MENU_BUTTON_ID = "open-menu-button";
const CLOSE_MENU_BUTTON_ID = "close-menu";
const DRAWER_BACKDROP_ID = "drawer-backdrop";

/**
 * Opens the mobile navigation drawer menu.
 * Adds transition effects and manages backdrop visibility.
 */
const openNavigationDrawer = () => {
  setDrawerVisibility({ drawerId: DRAWER_MENU_ID, open: true });
};

/**
 * Closes the mobile navigation drawer menu.
 * Removes transition effects and resets scroll position.
 */
const closeNavigationDrawer = () => {
  setDrawerVisibility({ drawerId: DRAWER_MENU_ID, open: false });
};

/**
 * Initializes the mobile menu open button click behavior.
 * @param {string} [buttonId="open-menu-button"] - Open menu button id
 */
export const initializeOpenMenuButton = (buttonId = OPEN_MENU_BUTTON_ID) => {
  const openMenuButton = document.getElementById(buttonId);
  if (!openMenuButton || openMenuButton.dataset.openMenuBound === "true") {
    return;
  }

  openMenuButton.addEventListener("click", openNavigationDrawer);
  openMenuButton.dataset.openMenuBound = "true";
};

/**
 * Initializes the mobile menu close controls.
 * @param {Object} [options] - Optional element id overrides
 * @param {string} [options.closeButtonId="close-menu"] - Close menu button id
 * @param {string} [options.backdropId="drawer-backdrop"] - Drawer backdrop id
 */
export const initializeCloseMenuControls = ({
  closeButtonId = CLOSE_MENU_BUTTON_ID,
  backdropId = DRAWER_BACKDROP_ID,
} = {}) => {
  initializeModalCloseHandlers({
    modalId: DRAWER_MENU_ID,
    triggerIds: [closeButtonId, backdropId],
    closeHandler: closeNavigationDrawer,
  });
};
