import {
  initializeCookiePreferencesButtons,
  initializeGlobalPopstateHandlers,
  initializePreviewModalHandlers,
  initializeToggleCheckboxes,
} from "/static/js/common/common.js";

/**
 * Initializes global common UI handlers used across pages.
 */
export const initializeCommonBootstrap = () => {
  initializeGlobalPopstateHandlers();
  initializeCookiePreferencesButtons();
  initializePreviewModalHandlers();
  initializeToggleCheckboxes();
};

initializeCommonBootstrap();
