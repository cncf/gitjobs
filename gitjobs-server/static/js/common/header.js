import { initializeButtonDropdown } from "/static/js/common/common.js";

let lifecycleListenersBound = false;

const getDropdownButton = () => document.getElementById("user-dropdown-button");

const getDropdownMenuId = () => {
  if (document.getElementById("dropdown-user")) {
    return "dropdown-user";
  }

  if (document.getElementById("user-dropdown")) {
    return "user-dropdown";
  }

  return "";
};

const getDropdownMenu = () => {
  const dropdownMenuId = getDropdownMenuId();
  if (!dropdownMenuId) {
    return null;
  }

  return document.getElementById(dropdownMenuId);
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
  bindLifecycleListeners();

  const button = getDropdownButton();
  const dropdownMenuId = getDropdownMenuId();
  const dropdown = getDropdownMenu();
  if (!button || !dropdown || !dropdownMenuId) {
    return;
  }

  initializeButtonDropdown({
    buttonId: "user-dropdown-button",
    dropdownId: dropdownMenuId,
    guardKey: `__gitjobsUserDropdownBound:${dropdownMenuId}`,
    closeOnItemClickSelector: "a",
  });

  button.setAttribute("aria-expanded", dropdown.classList.contains("hidden") ? "false" : "true");
  dropdown.setAttribute("aria-hidden", dropdown.classList.contains("hidden") ? "true" : "false");
};

initUserDropdown();
