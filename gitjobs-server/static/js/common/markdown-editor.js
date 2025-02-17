import { LitElement, html, createRef, ref } from "https://cdn.jsdelivr.net/gh/lit/dist@3/all/lit-all.min.js";

export class MarkdownEditor extends LitElement {
  static properties = {
    id: { type: String },
    name: { type: String },
    content: { type: String },
    required: { type: Boolean },
    onChange: { type: Function },
  };

  textareaRef = createRef();

  constructor() {
    super();
    this.id = "id";
    this.name = undefined;
    this.content = "";
    this.required = false;
    this.onChange = undefined;
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

    easyMDE.codemirror.on("change", () => {
      if (this.onChange) {
        this.onChange(easyMDE.value());
      }
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
          data-name="${this.name}"
          rows="3"
          class="absolute top-0 left-0 opacity-0"
          ?required=${this.required}
        >
          ${this.content}
        </textarea
        >
      </div>
    `;
  }
}
customElements.define("markdown-editor", MarkdownEditor);
