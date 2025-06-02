/**
 * Toggles the user dropdown menu visibility and manages event listeners.
 * Handles click-outside-to-close functionality.
 */
export const onClickDropdown = () => {
  const dropdownButton = document.getElementById("user-dropdown-button");
  const dropdownMenu = document.getElementById("dropdown-user");

  if (dropdownMenu) {
    const isHidden = dropdownMenu.classList.contains("hidden");

    if (isHidden) {
      dropdownMenu.classList.remove("hidden");

      const menuLinks = dropdownMenu.querySelectorAll("a");
      menuLinks.forEach((link) => {
        // Close dropdown actions when clicking on an action before loading the new page
        link.addEventListener("htmx:beforeOnLoad", () => {
          const menu = document.getElementById("dropdown-user");
          menu.classList.add("hidden");
        });
      });

      if (dropdownButton) {
        // Close dropdown actions when clicking outside
        document.addEventListener("click", (event) => {
          if (!dropdownMenu.contains(event.target) && !dropdownButton.contains(event.target)) {
            dropdownMenu.classList.add("hidden");
          }
        });
      }
    } else {
      dropdownMenu.classList.add("hidden");
      // TODO: Store and remove the actual event listener function
    }
  }
};
