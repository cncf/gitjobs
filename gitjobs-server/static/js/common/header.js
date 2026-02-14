let documentHandlersBound = false;
let lifecycleListenersBound = false;

const getDropdownButton = () => document.getElementById("user-dropdown-button");

const getDropdownMenu = () => {
  return document.getElementById("dropdown-user") || document.getElementById("user-dropdown");
};

const hideDropdown = () => {
  const button = getDropdownButton();
  const dropdown = getDropdownMenu();
  if (!dropdown) {
    return;
  }

  dropdown.classList.add("hidden");
  if (button) {
    button.setAttribute("aria-expanded", "false");
  }
};

const showDropdown = () => {
  const button = getDropdownButton();
  const dropdown = getDropdownMenu();
  if (!dropdown) {
    return;
  }

  dropdown.classList.remove("hidden");
  if (button) {
    button.setAttribute("aria-expanded", "true");
  }
};

const toggleDropdownVisibility = (event) => {
  const dropdown = getDropdownMenu();
  if (!dropdown) {
    return;
  }

  if (event && typeof event.stopPropagation === "function") {
    event.stopPropagation();
  }
  if (dropdown.classList.contains("hidden")) {
    showDropdown();
  } else {
    hideDropdown();
  }
};

const ensureDocumentHandlers = () => {
  if (documentHandlersBound) {
    return;
  }

  const handleDocumentClick = (event) => {
    const button = getDropdownButton();
    const dropdown = getDropdownMenu();
    if (!button || !dropdown) {
      return;
    }

    const clickedButton = button.contains(event.target);
    const clickedDropdown = dropdown.contains(event.target);
    if (!clickedButton && !clickedDropdown) {
      hideDropdown();
    }
  };

  const handleKeydown = (event) => {
    if (event.key !== "Escape") {
      return;
    }

    const button = getDropdownButton();
    const dropdown = getDropdownMenu();
    if (!button || !dropdown || dropdown.classList.contains("hidden")) {
      return;
    }

    hideDropdown();
    button.focus();
  };

  document.addEventListener("click", handleDocumentClick);
  document.addEventListener("keydown", handleKeydown);

  documentHandlersBound = true;
};

const bindLifecycleListeners = () => {
  if (lifecycleListenersBound) {
    return;
  }

  document.addEventListener("htmx:historyRestore", initUserDropdown);
  document.addEventListener("htmx:afterSwap", initUserDropdown);
  window.addEventListener("pageshow", () => initUserDropdown());

  lifecycleListenersBound = true;
};

export const initUserDropdown = () => {
  ensureDocumentHandlers();
  bindLifecycleListeners();

  const button = getDropdownButton();
  const dropdown = getDropdownMenu();
  if (!button || !dropdown) {
    return;
  }

  button.setAttribute("aria-expanded", dropdown.classList.contains("hidden") ? "false" : "true");

  if (!button.__gitjobsDropdownInitialized) {
    button.addEventListener("click", toggleDropdownVisibility);
    button.__gitjobsDropdownInitialized = true;
  }

  if (!dropdown.__gitjobsCloseOnLinkBound) {
    dropdown.addEventListener(
      "click",
      (event) => {
        const link = event.target.closest("a");
        if (!link) {
          return;
        }

        if (link.querySelector(".hx-spinner")) {
          return;
        }

        hideDropdown();
      },
      true,
    );
    dropdown.__gitjobsCloseOnLinkBound = true;
  }
};

initUserDropdown();
