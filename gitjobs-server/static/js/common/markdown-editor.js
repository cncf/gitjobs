import { html, createRef, ref } from "/static/vendor/js/lit-all.v3.2.1.min.js";
import { LitWrapper } from "/static/js/common/lit-wrapper.js";

/**
 * Markdown editor component using EasyMDE.
 * Provides rich text editing with markdown support.
 * @extends LitWrapper
 */
export class MarkdownEditor extends LitWrapper {
  /**
   * Component properties definition
   * @property {string} id - Editor ID attribute
   * @property {string} name - Form input name
   * @property {string} content - Initial markdown content
   * @property {boolean} required - Whether input is required
   * @property {Function} onChange - Callback for content changes
   * @property {boolean} mini - Use compact editor layout
   */
  static properties = {
    id: { type: String },
    name: { type: String },
    content: { type: String },
    required: { type: Boolean },
    onChange: { type: Function },
    mini: { type: Boolean },
  };

  /** @type {import('lit').Ref<HTMLTextAreaElement>} Reference to textarea */
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

    this._initEditor(textarea);
  }

  /**
   * Initializes the EasyMDE editor instance.
   * @param {HTMLTextAreaElement} textarea - The textarea element to enhance
   * @private
   */
  _initEditor(textarea) {
    const markdownEditor = new EasyMDE({
      element: textarea,
      forceSync: true,
      hideIcons: ["side-by-side", "fullscreen", "guide", "image", "code"],
      showIcons: ["code", "table", "undo", "redo", "horizontal-rule"],
      initialValue: this.content,
      status: false,
      previewClass: "markdown",
      // Fix for hidden textarea
      autoRefresh: { delay: 300 },
    });

    markdownEditor.codemirror.on("change", () => {
      if (this.onChange) {
        this.onChange(markdownEditor.value());
      }
    });

    // Show textarea to avoid console errors with required attribute
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
