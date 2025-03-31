import { html, createRef, ref } from "/static/vendor/js/lit-all.v3.2.1.min.js";
import { LitWrapper } from "/static/js/common/lit-wrapper.js";

export class MarkdownEditor extends LitWrapper {
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
