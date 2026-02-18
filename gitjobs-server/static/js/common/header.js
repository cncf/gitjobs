import { initializeButtonDropdown, isDashboardPath } from "/static/js/common/common.js";

let lifecycleListenersBound = false;
const USER_DROPDOWN_BUTTON_ID = "user-dropdown-button";
const USER_DROPDOWN_MENU_ID = "dropdown-user";

const shouldResetDashboardScroll = (event) => {
  if (!event) {
    return false;
  }

  const swapTarget = event.detail?.target || event.target;
  if (!swapTarget) {
    return false;
  }

  if (!isDashboardPath()) {
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
  bindLifecycleListeners();

  const button = document.getElementById(USER_DROPDOWN_BUTTON_ID);
  const dropdown = document.getElementById(USER_DROPDOWN_MENU_ID);
  if (!button || !dropdown) {
    return;
  }

  initializeButtonDropdown({
    buttonId: USER_DROPDOWN_BUTTON_ID,
    dropdownId: USER_DROPDOWN_MENU_ID,
    guardKey: "__gitjobsUserDropdownBound:dropdown-user",
    closeOnItemClickSelector: "a",
  });

  button.setAttribute("aria-expanded", dropdown.classList.contains("hidden") ? "false" : "true");
  dropdown.setAttribute("aria-hidden", dropdown.classList.contains("hidden") ? "true" : "false");
};

initUserDropdown();
