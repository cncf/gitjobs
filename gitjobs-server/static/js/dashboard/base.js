// Open menu view (only for mobile).
export const open = () => {
  const drawer = document.getElementById("drawer-menu");
  drawer.classList.add("transition-transform");
  drawer.classList.remove("-translate-x-full");
  drawer.dataset.open = "true";
  const backdrop = document.getElementById("drawer-backdrop");
  backdrop.classList.remove("hidden");
};

// Close menu view (only for mobile).
export const close = () => {
  const drawer = document.getElementById("drawer-menu");
  drawer.classList.add("-translate-x-full");
  drawer.classList.remove("transition-transform");
  drawer.dataset.open = "false";
  drawer.scrollTop = 0;
  const backdrop = document.getElementById("drawer-backdrop");
  backdrop.classList.add("hidden");
};
