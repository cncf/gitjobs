import { LitElement, html, createRef, ref } from "https://cdn.jsdelivr.net/gh/lit/dist@3/all/lit-all.min.js";

export class MarkdownEditor extends LitElement {
  static properties = {
    id: { type: String },
    content: { type: String },
    required: { type: Boolean },
  };

  textareaRef = createRef();

  constructor() {
    super();
    this.id = "id";
    this.content = "";
    this.required = false;
  }

  createRenderRoot() {
    // Disable shadow dom to use Tailwind CSS
    return this;
  }

  firstUpdated() {
    super.firstUpdated();

    const textarea = this.textareaRef.value;
    if (!textarea) {
      return;
    }

    const easyMDE = new EasyMDE({
      element: textarea,
      forceSync: true,
      hideIcons: ["side-by-side", "fullscreen", "guide", "heading", "image", "code"],
      showIcons: ["code", "table", "undo", "redo", "horizontal-rule"],
      initialValue: this.content,
      status: false,
      previewClass: "markdown",
    });

    if (this.required) {
      textarea.style.display = "block";
    }
  }

  render() {
    return html`
      <div class="relative">
        <textarea
          ${ref(this.textareaRef)}
          name="${this.id}"
          rows="3"
          class="absolute top-0 left-0 opacity-0"
          ?required=${this.required}
        >
${this.content}</textarea
        >
      </div>
    `;
  }
}
customElements.define("markdown-editor", MarkdownEditor);
