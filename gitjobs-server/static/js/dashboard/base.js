/**
 * Opens the mobile navigation drawer menu.
 * Adds transition effects and manages backdrop visibility.
 */
export const openNavigationDrawer = () => {
  const navigationDrawer = document.getElementById("drawer-menu");
  if (navigationDrawer) {
    navigationDrawer.classList.add("transition-transform");
    navigationDrawer.classList.remove("-translate-x-full");
    navigationDrawer.dataset.open = "true";
  }
  const backdrop = document.getElementById("drawer-backdrop");
  if (backdrop) {
    backdrop.classList.remove("hidden");
  }
};

/**
 * Closes the mobile navigation drawer menu.
 * Removes transition effects and resets scroll position.
 */
export const closeNavigationDrawer = () => {
  const navigationDrawer = document.getElementById("drawer-menu");
  if (navigationDrawer) {
    navigationDrawer.classList.add("-translate-x-full");
    navigationDrawer.classList.remove("transition-transform");
    navigationDrawer.dataset.open = "false";
    navigationDrawer.scrollTop = 0;
  }
  const backdrop = document.getElementById("drawer-backdrop");
  if (backdrop) {
    backdrop.classList.add("hidden");
  }
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
  const closeMenuButton = document.getElementById(closeButtonId);
  if (closeMenuButton && closeMenuButton.dataset.closeMenuBound !== "true") {
    closeMenuButton.addEventListener("click", closeNavigationDrawer);
    closeMenuButton.dataset.closeMenuBound = "true";
  }

  const backdropMenu = document.getElementById(backdropId);
  if (backdropMenu && backdropMenu.dataset.backdropMenuBound !== "true") {
    backdropMenu.addEventListener("click", closeNavigationDrawer);
    backdropMenu.dataset.backdropMenuBound = "true";
  }
};
