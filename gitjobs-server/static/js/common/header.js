/**
 * Handles the user dropdown menu toggle and its related interactions.
 *
 * - Shows or hides the user dropdown menu when the toggle button is clicked.
 * - Closes the dropdown when a dropdown link is clicked (before navigation).
 * - Closes the dropdown when clicking outside the menu or toggle button.
 */
let outsideClickHandler;

export const handleUserDropdownClick = () => {
  const dropdownToggleBtn = document.getElementById("user-dropdown-button");
  const userDropdownMenu = document.getElementById("dropdown-user");
  const isDropdownHidden = userDropdownMenu.classList.contains("hidden");

  if (isDropdownHidden) {
    userDropdownMenu.classList.remove("hidden");

    const dropdownLinks = userDropdownMenu.querySelectorAll("a");
    dropdownLinks.forEach((link) => {
      // Close dropdown when clicking on a link before loading the new page
      link.addEventListener("htmx:beforeOnLoad", () => {
        dropdownMenu.classList.add("hidden");
      });
    });

    outsideClickHandler = (event) => {
      if (!userDropdownMenu.contains(event.target) && !dropdownToggleBtn.contains(event.target)) {
        userDropdownMenu.classList.add("hidden");
        document.removeEventListener("click", outsideClickHandler);
      }
    };
    document.addEventListener("click", outsideClickHandler);
  } else {
    userDropdownMenu.classList.add("hidden");
    if (outsideClickHandler) {
      document.removeEventListener("click", outsideClickHandler);
    }
  }
};
