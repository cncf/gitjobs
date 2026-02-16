import { handleHtmxResponse } from "/static/js/common/alerts.js";

/**
 * Initializes image upload form behavior.
 * @param {Object} options - Element identifiers
 * @param {string} options.hiddenInputId - Hidden input id where image id is stored
 * @param {string} [options.formId="images-form"] - Images form id
 * @param {string} [options.cleanButtonId="clean-image"] - Remove image button id
 * @param {string} [options.imageId="image"] - Preview image id
 * @param {string} [options.placeholderId="placeholder-image"] - Placeholder image id
 */
export const initializeImagesForm = ({
  hiddenInputId,
  formId = "images-form",
  cleanButtonId = "clean-image",
  imageId = "image",
  placeholderId = "placeholder-image",
}) => {
  const cleanImage = document.getElementById(cleanButtonId);
  const imagesForm = document.getElementById(formId);
  const image = document.getElementById(imageId);
  const inputHidden = hiddenInputId ? document.getElementById(hiddenInputId) : null;
  const placeholderImage = document.getElementById(placeholderId);

  if (!cleanImage || !imagesForm || !image || !inputHidden || !placeholderImage) {
    return;
  }

  if (imagesForm.dataset.imagesFormBound === "true") {
    return;
  }

  imagesForm.addEventListener("htmx:afterRequest", (event) => {
    const isSuccessful = handleHtmxResponse({
      xhr: event.detail.xhr,
      successMessage: "Image added successfully.",
      errorMessage:
        "Something went wrong adding the image. Please try again later." +
        '<br /><br /><div class="text-sm text-stone-500">' +
        "Images must be at least 400x400, preferably in square format. " +
        "Maximum file size: 1MB. Formats supported: SVG, PNG, JPEG, GIF, WEBP and TIFF." +
        "</div>",
      errorWithHtml: true,
      treatUnprocessableAsGenericError: true,
    });

    if (!isSuccessful) {
      return;
    }

    const imageIdValue = event.detail.xhr.response;
    inputHidden.value = imageIdValue;
    image.setAttribute("src", `/dashboard/images/${imageIdValue}/small`);
    image.classList.remove("hidden");
    placeholderImage.classList.add("hidden");
    cleanImage.removeAttribute("disabled");
  });

  cleanImage.addEventListener("click", () => {
    inputHidden.value = "";
    cleanImage.disabled = true;
    placeholderImage.classList.remove("hidden");
    image.setAttribute("src", "");
    image.classList.add("hidden");
  });

  imagesForm.dataset.imagesFormBound = "true";
};

/**
 * Initializes all image forms rendered in the DOM.
 * @param {Document|HTMLElement} [root=document] - Root where image forms are searched
 */
export const initializeImagesForms = (root = document) => {
  const selector = '[data-images-form="true"]';
  const imageForms = [];

  if (root instanceof Element && root.matches(selector)) {
    imageForms.push(root);
  }

  if (typeof root.querySelectorAll === "function") {
    imageForms.push(...root.querySelectorAll(selector));
  }

  imageForms.forEach((imageForm) => {
    initializeImagesForm({
      hiddenInputId: imageForm.dataset.hiddenInputId || "",
      formId: imageForm.id || "images-form",
      cleanButtonId: imageForm.dataset.cleanButtonId || "clean-image",
      imageId: imageForm.dataset.imageId || "image",
      placeholderId: imageForm.dataset.placeholderId || "placeholder-image",
    });
  });

  if (document.__gitjobsImagesFormsLifecycleBound) {
    return;
  }

  document.addEventListener("htmx:afterSwap", (event) => {
    initializeImagesForms(event.target);
  });
  document.addEventListener("htmx:historyRestore", () => {
    initializeImagesForms(document);
  });
  window.addEventListener("pageshow", () => {
    initializeImagesForms(document);
  });

  document.__gitjobsImagesFormsLifecycleBound = true;
};

initializeImagesForms();
