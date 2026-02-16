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
  dropdown.setAttribute("aria-hidden", "true");
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
  dropdown.setAttribute("aria-hidden", "false");
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

const shouldResetDashboardScroll = (event) => {
  if (!event) {
    return false;
  }

  const swapTarget = event.detail?.target || event.target;
  if (!swapTarget) {
    return false;
  }

  const path = window.location?.pathname || "";
  if (!path.startsWith("/dashboard/")) {
    return false;
  }

  return swapTarget === document.body || swapTarget.id === "dashboard-content";
};

const scrollToTopOnDashboardSwap = (event) => {
  if (!shouldResetDashboardScroll(event) || typeof window.scrollTo !== "function") {
    return;
  }

  window.scrollTo({ top: 0, behavior: "auto" });
};

const bindLifecycleListeners = () => {
  if (lifecycleListenersBound) {
    return;
  }

  document.addEventListener("htmx:historyRestore", initUserDropdown);
  document.addEventListener("htmx:afterSwap", initUserDropdown);
  document.addEventListener("htmx:afterSwap", scrollToTopOnDashboardSwap);
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
  dropdown.setAttribute("aria-hidden", dropdown.classList.contains("hidden") ? "true" : "false");

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
