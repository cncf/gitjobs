import {
  cleanInputField,
  closeFiltersDrawer,
  openFiltersDrawer,
  resetForm,
  searchOnEnter,
  triggerActionOnForm,
} from "/static/js/jobboard/filters.js";
import { shouldDisplayJobModal, toggleModalVisibility } from "/static/js/common/common.js";
import { copyEmbedCodeToClipboard, renderEmbedCode } from "/static/js/jobboard/job_section.js";

/**
 * Initializes jobboard explore page controls and modals.
 */
export const initializeJobboardExplore = () => {
  const openFiltersButton = document.getElementById("open-filters");
  if (openFiltersButton && openFiltersButton.dataset.boundOpenFilters !== "true") {
    openFiltersButton.addEventListener("click", openFiltersDrawer);
    openFiltersButton.dataset.boundOpenFilters = "true";
  }

  const closeFiltersButton = document.getElementById("close-filters");
  if (closeFiltersButton && closeFiltersButton.dataset.boundCloseFilters !== "true") {
    closeFiltersButton.addEventListener("click", closeFiltersDrawer);
    closeFiltersButton.dataset.boundCloseFilters = "true";
  }

  const filtersBackdrop = document.getElementById("drawer-backdrop");
  if (filtersBackdrop && filtersBackdrop.dataset.boundBackdropFilters !== "true") {
    filtersBackdrop.addEventListener("click", closeFiltersDrawer);
    filtersBackdrop.dataset.boundBackdropFilters = "true";
  }

  const sortSelectDesktop = document.getElementById("sort-desktop");
  if (sortSelectDesktop && sortSelectDesktop.dataset.boundSortDesktop !== "true") {
    sortSelectDesktop.addEventListener("change", () => {
      triggerActionOnForm("desktop-jobs-form", "submit");
    });
    sortSelectDesktop.dataset.boundSortDesktop = "true";
  }

  const sortSelectMobile = document.getElementById("sort-mobile");
  if (sortSelectMobile && sortSelectMobile.dataset.boundSortMobile !== "true") {
    sortSelectMobile.addEventListener("change", () => {
      triggerActionOnForm("mobile-jobs-form", "submit");
    });
    sortSelectMobile.dataset.boundSortMobile = "true";
  }

  const searchInput = document.getElementById("searchbar");
  if (searchInput && searchInput.dataset.boundSearchEnter !== "true") {
    searchInput.addEventListener("keydown", (event) => searchOnEnter(event, "desktop-jobs-form"));
    searchInput.dataset.boundSearchEnter = "true";
  }

  const cleanSearchButton = document.getElementById("clean-search-jobs");
  if (cleanSearchButton && cleanSearchButton.dataset.boundCleanSearch !== "true") {
    cleanSearchButton.addEventListener("click", () => cleanInputField("searchbar", "desktop-jobs-form"));
    cleanSearchButton.dataset.boundCleanSearch = "true";
  }

  const searchJobsButton = document.getElementById("search-jobs-btn");
  if (searchJobsButton && searchJobsButton.dataset.boundSearchDesktop !== "true") {
    searchJobsButton.addEventListener("click", () =>
      triggerActionOnForm("desktop-jobs-form", "submit", true),
    );
    searchJobsButton.dataset.boundSearchDesktop = "true";
  }

  const searchJobsMobileButton = document.getElementById("search-jobs-btn-mobile");
  if (searchJobsMobileButton && searchJobsMobileButton.dataset.boundSearchMobile !== "true") {
    searchJobsMobileButton.addEventListener("click", () =>
      triggerActionOnForm("mobile-jobs-form", "submit", true),
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

  const resetDesktopFilters = document.getElementById("reset-desktop-filters");
  if (resetDesktopFilters && resetDesktopFilters.dataset.boundResetDesktop !== "true") {
    resetDesktopFilters.addEventListener("click", () => resetForm("desktop-jobs-form"));
    resetDesktopFilters.dataset.boundResetDesktop = "true";
  }

  const resetMobileFilters = document.getElementById("reset-mobile-filters");
  if (resetMobileFilters && resetMobileFilters.dataset.boundResetMobile !== "true") {
    resetMobileFilters.addEventListener("click", () => resetForm("mobile-jobs-form"));
    resetMobileFilters.dataset.boundResetMobile = "true";
  }

  const embedButtons = document.querySelectorAll("[data-embed]");
  embedButtons.forEach((button) => {
    if (button.dataset.boundEmbedOpen === "true") {
      return;
    }

    button.addEventListener("click", () => {
      renderEmbedCode();
      toggleModalVisibility("embed-modal", "open");

      const device = button.getAttribute("data-device");
      if (device === "mobile") {
        closeFiltersDrawer();
      }
    });

    button.dataset.boundEmbedOpen = "true";
  });

  const copyEmbedCodeButton = document.getElementById("copy-embed-code");
  if (copyEmbedCodeButton && copyEmbedCodeButton.dataset.boundCopyEmbed !== "true") {
    copyEmbedCodeButton.addEventListener("click", () => {
      copyEmbedCodeToClipboard("embed-code");
    });
    copyEmbedCodeButton.dataset.boundCopyEmbed = "true";
  }

  const closeEmbedModal = document.getElementById("close-embed-modal");
  if (closeEmbedModal && closeEmbedModal.dataset.boundCloseEmbed !== "true") {
    closeEmbedModal.addEventListener("click", () => {
      toggleModalVisibility("embed-modal", "close");
    });
    closeEmbedModal.dataset.boundCloseEmbed = "true";
  }

  const backdropEmbedModal = document.getElementById("backdrop-embed-modal");
  if (backdropEmbedModal && backdropEmbedModal.dataset.boundBackdropEmbed !== "true") {
    backdropEmbedModal.addEventListener("click", () => {
      toggleModalVisibility("embed-modal", "close");
    });
    backdropEmbedModal.dataset.boundBackdropEmbed = "true";
  }

  shouldDisplayJobModal(true);
};
