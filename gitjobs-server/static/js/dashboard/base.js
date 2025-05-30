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
