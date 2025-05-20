import { LitElement } from "/static/vendor/js/lit-all.v3.2.1.min.js";

/**
 * LitWrapper class extends LitElement to disable shadow DOM.
 * This allows usage of global styles like Tailwind CSS.
 */
export class LitWrapper extends LitElement {
  /**
   * Overrides LitElement's render root creation.
   * Disables shadow DOM so styles apply globally.
   * Clears innerHTML if children exist to avoid duplicate content.
   * @returns {HTMLElement} The element itself as the render root.
   */
  createRenderRoot() {
    if (this.children.length === 0) {
      // Disable shadow dom to use Tailwind CSS
      return this;
    } else {
      // Remove previous content when re-rendering full component
      this.innerHTML = "";
      // Disable shadow dom to use Tailwind CSS
      return this;
    }
  }
}
