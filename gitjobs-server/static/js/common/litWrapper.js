import { LitElement } from "https://cdn.jsdelivr.net/gh/lit/dist@3/core/lit-core.min.js";

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
