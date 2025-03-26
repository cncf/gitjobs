export const onClickDropdown = () => {
  const dropdownButton = document.getElementById("user-dropdown-button");
  const dropdown = document.getElementById("dropdown-user");
  const isHidden = dropdown.classList.contains("hidden");

  if (isHidden) {
    dropdown.classList.remove("hidden");

    const anchors = dropdown.querySelectorAll("a");
    anchors.forEach((anchor) => {
      // Close dropdown actions when clicking on an action before loading the new page
      anchor.addEventListener("htmx:beforeOnLoad", () => {
        const dropdown = document.getElementById("dropdown-user");
        dropdown.classList.add("hidden");
      });
    });

    // Close dropdown actions when clicking outside
    document.addEventListener("click", (event) => {
      if (!dropdown.contains(event.target) && !dropdownButton.contains(event.target)) {
        dropdown.classList.add("hidden");
      }
    });
  } else {
    dropdown.classList.add("hidden");
    // Remove event listener when dropdown is closed
    document.removeEventListener("click", () => {});
  }
};
