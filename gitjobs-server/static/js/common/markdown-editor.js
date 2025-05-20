import { html, createRef, ref } from "/static/vendor/js/lit-all.v3.2.1.min.js";
import { LitWrapper } from "/static/js/common/lit-wrapper.js";

/**
 * MarkdownEditor web component for editing markdown content with EasyMDE and Lit.
 * Extends LitWrapper and uses Lit for rendering.

 * @class MarkdownEditor
 * @property {string} id - Unique identifier for the editor.
 * @property {string} name - Name attribute for the textarea.
 * @property {string} content - Initial markdown content.
 * @property {boolean} required - If true, textarea is required.
 * @property {function} onContentChange - Callback for content changes.
 * @property {boolean} mini - If true, renders a compact editor.
 */
export class MarkdownEditor extends LitWrapper {
  static properties = {
    id: { type: String },
    name: { type: String },
    content: { type: String },
    required: { type: Boolean },
    onContentChange: { type: Function },
    mini: { type: Boolean },
  };

  /**
   * Reference to the textarea element.
   */
  textareaRef = createRef();

  /**
   * Initializes default property values.
   */
  constructor() {
    super();
    this.id = "id";
    this.name = undefined;
    this.content = "";
    this.required = false;
    this.onContentChange = undefined;
    this.mini = false;
  }

  /**
   * Lit lifecycle: called after the component's first render.
   * Initializes the markdown editor.
   */
  firstUpdated() {
    super.firstUpdated();
    const textarea = this.textareaRef.value;
    if (!textarea) {
      return;
    }
    this._initializeMarkdownEditor(textarea);
  }

  /**
   * Initializes EasyMDE markdown editor on the textarea.
   * Sets up change event to call onContentChange callback if provided.
   * @param {HTMLTextAreaElement} textarea - The textarea to enhance.
   */
  _initializeMarkdownEditor(textarea) {
    const easyMDE = new EasyMDE({
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

    easyMDE.codemirror.on("change", () => {
      if (this.onContentChange) {
        this.onContentChange(easyMDE.value());
      }
    });

    // Ensure textarea is visible to avoid errors if required attribute is set
    textarea.style.display = "block";
  }

  /**
   * Renders the markdown editor container and textarea.
   * @returns {import("lit").TemplateResult} Lit template for the editor.
   */
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
/**
 * Registers the MarkdownEditor component as a custom element.
 */
customElements.define("markdown-editor", MarkdownEditor);
