import { LitElement, html, createRef, ref } from "https://cdn.jsdelivr.net/gh/lit/dist@3/all/lit-all.min.js";

export class MarkdownEditor extends LitElement {
  static properties = {
    id: { type: String },
    name: { type: String },
    content: { type: String },
    required: { type: Boolean },
    onChange: { type: Function },
    mini: { type: Boolean },
  };

  textareaRef = createRef();

  constructor() {
    super();
    this.id = "id";
    this.name = undefined;
    this.content = "";
    this.required = false;
    this.onChange = undefined;
    this.mini = false;
  }

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
      // Fix for hidden textarea
      autoRefresh: { delay: 300 },
    });

    easyMDE.codemirror.on("change", () => {
      if (this.onChange) {
        this.onChange(easyMDE.value());
      }
    });

    // Update display of textare to avoid console errors if required attribute is set
    textarea.style.display = "block";
  }

  render() {
    return html`
      <div class="relative text-sm/6 ${this.mini ? "mini" : ""}">
        <textarea
          ${ref(this.textareaRef)}
          name="${this.id}"
          class="absolute top-0 left-0 opacity-0 p-0"
          ?required=${this.required}
        ></textarea>
      </div>
    `;
  }
}
customElements.define("markdown-editor", MarkdownEditor);
