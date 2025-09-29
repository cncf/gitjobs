import { LitElement } from "/static/vendor/js/lit-all.v3.3.1.min.js";

/**
 * Base wrapper class for Lit components that disables shadow DOM.
 * Allows components to use global Tailwind CSS styles.
 * @extends LitElement
 */
export class LitWrapper extends LitElement {
  /**
   * Creates the render root for the component.
   * Disables shadow DOM to enable global CSS access.
   * Clears innerHTML when re-rendering to prevent duplicate content.
   * @returns {LitWrapper} The component instance as render root
   */
  createRenderRoot() {
    if (this.children.length === 0) {
      // Disable shadow DOM to use Tailwind CSS
      return this;
    } else {
      // Remove previous content when re-rendering full component
      this.innerHTML = "";
      // Disable shadow DOM to use Tailwind CSS
      return this;
    }
  }
}
