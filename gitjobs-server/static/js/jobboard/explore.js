import {
  cleanInputField,
  closeFiltersDrawer,
  openFiltersDrawer,
  resetForm,
  searchOnEnter,
  triggerActionOnForm,
} from "/static/js/jobboard/filters.js";
import {
  bindHtmxAfterRequestOnce,
  initializeModalCloseHandlers,
  shouldDisplayJobModal,
  toggleModalVisibility,
} from "/static/js/common/common.js";
import { copyEmbedCodeToClipboard, renderEmbedCode } from "/static/js/jobboard/job_section.js";

const DESKTOP_JOBS_FORM_ID = "desktop-jobs-form";
const MOBILE_JOBS_FORM_ID = "mobile-jobs-form";
const SEARCHBAR_ID = "searchbar";
const EMBED_MODAL_ID = "embed-modal";
const DRAWER_FILTERS_ID = "drawer-filters";
const OPEN_FILTERS_BUTTON_ID = "open-filters";
const CLOSE_FILTERS_BUTTON_ID = "close-filters";
const DRAWER_BACKDROP_ID = "drawer-backdrop";
const SORT_DESKTOP_ID = "sort-desktop";
const SORT_MOBILE_ID = "sort-mobile";
const CLEAN_SEARCH_JOBS_BUTTON_ID = "clean-search-jobs";
const SEARCH_JOBS_BUTTON_ID = "search-jobs-btn";
const SEARCH_JOBS_MOBILE_BUTTON_ID = "search-jobs-btn-mobile";
const RESET_DESKTOP_FILTERS_ID = "reset-desktop-filters";
const RESET_MOBILE_FILTERS_ID = "reset-mobile-filters";
const COPY_EMBED_CODE_BUTTON_ID = "copy-embed-code";
const EMBED_CODE_ID = "embed-code";
const CLOSE_EMBED_MODAL_BUTTON_ID = "close-embed-modal";
const BACKDROP_EMBED_MODAL_ID = "backdrop-embed-modal";

/**
 * Initializes jobboard explore page controls and modals.
 */
export const initializeJobboardExplore = () => {
  bindHtmxAfterRequestOnce({
    selector: `#${DESKTOP_JOBS_FORM_ID}, #${MOBILE_JOBS_FORM_ID}`,
    handler: () => {
      if (typeof window.scrollTo === "function") {
        window.scrollTo({ top: 0, behavior: "auto" });
      }
    },
    boundAttribute: "jobboardFiltersAfterRequestScrollBound",
  });

  const openFiltersButton = document.getElementById(OPEN_FILTERS_BUTTON_ID);
  if (openFiltersButton && openFiltersButton.dataset.boundOpenFilters !== "true") {
    openFiltersButton.addEventListener("click", openFiltersDrawer);
    openFiltersButton.dataset.boundOpenFilters = "true";
  }

  initializeModalCloseHandlers({
    modalId: DRAWER_FILTERS_ID,
    triggerIds: [CLOSE_FILTERS_BUTTON_ID, DRAWER_BACKDROP_ID],
    closeHandler: closeFiltersDrawer,
  });

  const sortSelectDesktop = document.getElementById(SORT_DESKTOP_ID);
  if (sortSelectDesktop && sortSelectDesktop.dataset.boundSortDesktop !== "true") {
    sortSelectDesktop.addEventListener("change", () => {
      triggerActionOnForm(DESKTOP_JOBS_FORM_ID, "submit");
    });
    sortSelectDesktop.dataset.boundSortDesktop = "true";
  }

  const sortSelectMobile = document.getElementById(SORT_MOBILE_ID);
  if (sortSelectMobile && sortSelectMobile.dataset.boundSortMobile !== "true") {
    sortSelectMobile.addEventListener("change", () => {
      triggerActionOnForm(MOBILE_JOBS_FORM_ID, "submit");
    });
    sortSelectMobile.dataset.boundSortMobile = "true";
  }

  const searchInput = document.getElementById(SEARCHBAR_ID);
  if (searchInput && searchInput.dataset.boundSearchEnter !== "true") {
    searchInput.addEventListener("keydown", (event) => searchOnEnter(event, DESKTOP_JOBS_FORM_ID));
    searchInput.dataset.boundSearchEnter = "true";
  }

  const cleanSearchButton = document.getElementById(CLEAN_SEARCH_JOBS_BUTTON_ID);
  if (cleanSearchButton && cleanSearchButton.dataset.boundCleanSearch !== "true") {
    cleanSearchButton.addEventListener("click", () => cleanInputField(SEARCHBAR_ID, DESKTOP_JOBS_FORM_ID));
    cleanSearchButton.dataset.boundCleanSearch = "true";
  }

  const searchJobsButton = document.getElementById(SEARCH_JOBS_BUTTON_ID);
  if (searchJobsButton && searchJobsButton.dataset.boundSearchDesktop !== "true") {
    searchJobsButton.addEventListener("click", () =>
      triggerActionOnForm(DESKTOP_JOBS_FORM_ID, "submit", true),
    );
    searchJobsButton.dataset.boundSearchDesktop = "true";
  }

  const searchJobsMobileButton = document.getElementById(SEARCH_JOBS_MOBILE_BUTTON_ID);
  if (searchJobsMobileButton && searchJobsMobileButton.dataset.boundSearchMobile !== "true") {
    searchJobsMobileButton.addEventListener("click", () =>
      triggerActionOnForm(MOBILE_JOBS_FORM_ID, "submit", true),
    );
    searchJobsMobileButton.dataset.boundSearchMobile = "true";
  }

  const formItems = document.querySelectorAll("[data-trigger-form]");
  formItems.forEach((item) => {
    if (item.dataset.boundTriggerForm === "true") {
      return;
    }

    item.addEventListener("change", () => {
      if (item.tagName === "SELECT") {
        item.blur();
      }

      const form = item.getAttribute("form");
      if (form) {
        triggerActionOnForm(form, "submit");
      }
    });

    item.dataset.boundTriggerForm = "true";
  });

  const foundationSelects = document.querySelectorAll('select[id$="-foundation"]');
  foundationSelects.forEach((foundationSelect) => {
    if (foundationSelect.dataset.boundFoundationSelect === "true") {
      return;
    }

    foundationSelect.addEventListener("change", (event) => {
      if (event.target.value === "") {
        return;
      }

      const searchProjects = document.getElementsByTagName("search-projects");
      for (let i = 0; i < searchProjects.length; i++) {
        searchProjects[i].cleanSelected();
      }
    });

    foundationSelect.dataset.boundFoundationSelect = "true";
  });

  const resetDesktopFilters = document.getElementById(RESET_DESKTOP_FILTERS_ID);
  if (resetDesktopFilters && resetDesktopFilters.dataset.boundResetDesktop !== "true") {
    resetDesktopFilters.addEventListener("click", () => resetForm(DESKTOP_JOBS_FORM_ID));
    resetDesktopFilters.dataset.boundResetDesktop = "true";
  }

  const resetMobileFilters = document.getElementById(RESET_MOBILE_FILTERS_ID);
  if (resetMobileFilters && resetMobileFilters.dataset.boundResetMobile !== "true") {
    resetMobileFilters.addEventListener("click", () => resetForm(MOBILE_JOBS_FORM_ID));
    resetMobileFilters.dataset.boundResetMobile = "true";
  }

  const embedButtons = document.querySelectorAll("[data-embed]");
  embedButtons.forEach((button) => {
    if (button.dataset.boundEmbedOpen === "true") {
      return;
    }

    button.addEventListener("click", () => {
      renderEmbedCode();
      toggleModalVisibility(EMBED_MODAL_ID, "open");

      const device = button.getAttribute("data-device");
      if (device === "mobile") {
        closeFiltersDrawer();
      }
    });

    button.dataset.boundEmbedOpen = "true";
  });

  const copyEmbedCodeButton = document.getElementById(COPY_EMBED_CODE_BUTTON_ID);
  if (copyEmbedCodeButton && copyEmbedCodeButton.dataset.boundCopyEmbed !== "true") {
    copyEmbedCodeButton.addEventListener("click", () => {
      copyEmbedCodeToClipboard(EMBED_CODE_ID);
    });
    copyEmbedCodeButton.dataset.boundCopyEmbed = "true";
  }

  initializeModalCloseHandlers({
    modalId: EMBED_MODAL_ID,
    triggerIds: [CLOSE_EMBED_MODAL_BUTTON_ID, BACKDROP_EMBED_MODAL_ID],
  });

  shouldDisplayJobModal(true);
};
