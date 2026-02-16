import { initializeModalCloseHandlers, setDrawerVisibility } from "/static/js/common/common.js";

/**
 * Opens the mobile navigation drawer menu.
 * Adds transition effects and manages backdrop visibility.
 */
export const openNavigationDrawer = () => {
  setDrawerVisibility({ drawerId: "drawer-menu", open: true });
};

/**
 * Closes the mobile navigation drawer menu.
 * Removes transition effects and resets scroll position.
 */
export const closeNavigationDrawer = () => {
  setDrawerVisibility({ drawerId: "drawer-menu", open: false });
};

/**
 * Initializes the mobile menu open button click behavior.
 * @param {string} [buttonId="open-menu-button"] - Open menu button id
 */
export const initializeOpenMenuButton = (buttonId = "open-menu-button") => {
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
  closeButtonId = "close-menu",
  backdropId = "drawer-backdrop",
} = {}) => {
  initializeModalCloseHandlers({
    modalId: "drawer-menu",
    triggerIds: [closeButtonId, backdropId],
    closeHandler: closeNavigationDrawer,
  });
};
