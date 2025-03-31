import { LitElement } from "/static/vendor/js/lit-all.v3.2.1.min.js";

export class LitWrapper extends LitElement {
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
