import { html, repeat } from "/static/vendor/js/lit-all.v3.3.1.min.js";
import { isObjectEmpty } from "/static/js/common/common.js";
import { LitWrapper } from "/static/js/common/lit-wrapper.js";

/**
 * Component for managing education entries in job seeker profile.
 * Supports adding, removing, and reordering education items.
 * @extends LitWrapper
 */
export class EducationSection extends LitWrapper {
  /**
   * Component properties definition
   * @property {Array} education - List of education entries
   * Each entry contains:
   *  - id: Unique identifier
   *  - title: Degree or qualification title
   *  - educational_institution: Name of the institution
   *  - description: Additional details (markdown format)
   *  - start_date: Start date of education
   *  - end_date: End date of education
   */
  static properties = {
    education: { type: Array },
  };

  constructor() {
    super();
    this.education = [];
  }

  connectedCallback() {
    super.connectedCallback();
    this._initializeEducationIds();
  }

  /**
   * Assigns unique IDs to education entries.
   * Creates initial entry if none exist.
   * @private
   */
  _initializeEducationIds() {
    if (this.education === null) {
      this.education = [this._getData()];
    } else {
      this.education = this.education.map((item, index) => {
        return { ...item, id: index };
      });
    }
  }

  /**
   * Creates a new empty education data object.
   * @returns {Object} Empty education entry
   * @private
   */
  _getData = () => {
    let item = {
      id: this.education ? this.education.length : 0,
      title: "",
      educational_institution: "",
      description: "",
      start_date: "",
      end_date: "",
    };

    return item;
  };

  /**
   * Adds a new education entry at specified index.
   * @param {number} index - Position to insert new entry
   * @private
   */
  _addEducationItem(index) {
    const currentEducation = [...this.education];
    currentEducation.splice(index, 0, this._getData());

    this.education = currentEducation;
  }

  /**
   * Removes education entry at specified index.
   * Ensures at least one empty entry remains.
   * @param {number} index - Position of entry to remove
   * @private
   */
  _removeEducationItem(index) {
    const tmpEducation = this.education.filter((_, i) => i !== index);
    // If there are no more education items, add a new one
    this.education = tmpEducation.length === 0 ? [this._getData()] : tmpEducation;
  }

  /**
   * Updates education data at specified index.
   * @param {Object} data - Updated education data
   * @param {number} index - Index of entry to update
   * @private
   */
  _onDataChange = (data, index) => {
    this.education[index] = data;
  };

  /**
   * Renders an education entry with controls.
   * @param {number} index - Entry index
   * @param {Object} education - Education data
   * @returns {import('lit').TemplateResult} Entry template
   * @private
   */
  _getEducationForm(index, education) {
    const hasSingleEducationItem = this.education.length === 1;

    return html`<div class="mt-10">
      <div class="flex w-full xl:w-2/3">
        <div class="flex flex-col space-y-3 me-3">
          <div>
            <button
              @click=${() => this._addEducationItem(index)}
              type="button"
              class="cursor-pointer p-2 border border-stone-200 hover:bg-stone-100 rounded-full"
              title="Add above"
            >
              <div class="svg-icon size-4 icon-plus_top bg-stone-600"></div>
            </button>
          </div>
          <div>
            <button
              @click=${() => this._addEducationItem(index + 1)}
              type="button"
              class="cursor-pointer p-2 border border-stone-200 hover:bg-stone-100 rounded-full"
              title="Add below"
            >
              <div class="svg-icon size-4 icon-plus_bottom bg-stone-600"></div>
            </button>
          </div>
          <div>
            <button
              @click=${() => this._removeEducationItem(index)}
              type="button"
              class="cursor-pointer p-2 border border-stone-200 hover:bg-stone-100 rounded-full"
              title="${hasSingleEducationItem ? "Clean" : "Delete"}"
            >
              <div
                class="svg-icon size-4 icon-${hasSingleEducationItem ? "eraser" : "trash"} bg-stone-600"
              ></div>
            </button>
          </div>
        </div>
        <education-item
          .data=${education}
          .index=${index}
          .onDataChange=${this._onDataChange}
          class="w-full"
        ></education-item>
      </div>
    </div>`;
  }

  render() {
    return html` <div class="text-sm/6 text-stone-500">
        Indicate your education. You can add additional entries by clicking on the
        <span class="font-semibold">+</span> buttons on the left of the card (
        <div class="inline-block svg-icon size-4 icon-plus_top bg-stone-600 relative -bottom-[2px]"></div>
        to add the new entry above,
        <div class="inline-block svg-icon size-4 icon-plus_bottom bg-stone-600 relative -bottom-[2px]"></div>
        to add it below). Entries will be displayed in the order provided.
      </div>
      <div id="education-section">
        ${repeat(
          this.education,
          (e) => e.id,
          (e, index) => this._getEducationForm(index, e),
        )}
      </div>`;
  }
}
customElements.define("education-section", EducationSection);

/**
 * Individual education entry component.
 * Handles form inputs and validation for a single education item.
 * @extends LitWrapper
 */
class EducationItem extends LitWrapper {
  /**
   * Component properties definition
   * @property {Object} data - Education entry data
   * Each entry contains:
   *  - id: Unique identifier
   *  - title: Degree or qualification title
   *  - educational_institution: Name of the institution
   *  - description: Additional details (markdown format)
   *  - start_date: Start date of education
   *  - end_date: End date of education
   * @property {number} index - Position of the entry in the list
   * @property {boolean} isObjectEmpty - Indicates if the data object is empty
   * @property {Function} onDataChange - Callback function to notify parent component of changes
   */
  static properties = {
    data: { type: Object },
    index: { type: Number },
    isObjectEmpty: { type: Boolean },
    onDataChange: { type: Function },
  };

  constructor() {
    super();
    this.data = {
      id: 0,
      title: "",
      educational_institution: "",
      description: "",
      start_date: "",
      end_date: "",
    };
    this.index = 0;
    this.isObjectEmpty = true;
    this.onDataChange = () => {};
  }

  connectedCallback() {
    super.connectedCallback();
    this.isObjectEmpty = isObjectEmpty(this.data);
  }

  /**
   * Handles input field changes.
   * @param {Event} event - Input event
   * @private
   */
  _onInputChange = (event) => {
    const value = event.target.value;
    const name = event.target.dataset.name;

    this.data[name] = value;
    this.isObjectEmpty = isObjectEmpty(this.data);
    this.onDataChange(this.data, this.index);
  };

  /**
   * Handles markdown editor changes.
   * @param {string} value - Updated markdown content
   * @private
   */
  _onTextareaChange = (value) => {
    this.data.description = value;
    this.isObjectEmpty = isObjectEmpty(this.data);
    this.onDataChange(this.data, this.index);
  };

  render() {
    return html` <div
      class="grid grid-cols-1 gap-x-6 gap-y-8 sm:grid-cols-6 border-2 border-stone-200 border-dashed p-8 rounded-lg bg-stone-50/25 w-full"
    >
      <div class="col-span-3">
        <label class="form-label"> Title <span class="asterisk">*</span> </label>
        <div class="mt-2">
          <input
            @input=${(e) => this._onInputChange(e)}
            data-name="title"
            type="text"
            name="education[${this.index}][title]"
            class="input-primary"
            value="${this.data.title}"
            autocomplete="off"
            autocorrect="off"
            autocapitalize="off"
            spellcheck="false"
            ?required=${!this.isObjectEmpty}
          />
        </div>
      </div>

      <div class="col-span-3">
        <label class="form-label"> Educational institution <span class="asterisk">*</span> </label>
        <div class="mt-2">
          <input
            @input=${(e) => this._onInputChange(e)}
            data-name="educational_institution"
            type="text"
            name="education[${this.index}][educational_institution]"
            class="input-primary"
            value="${this.data.educational_institution}"
            autocomplete="off"
            autocorrect="off"
            autocapitalize="off"
            spellcheck="false"
            ?required=${!this.isObjectEmpty}
          />
        </div>
      </div>

      <div class="col-span-full">
        <label for="summary" class="form-label"> Description <span class="asterisk">*</span> </label>
        <div class="mt-2">
          <markdown-editor
            id="education[${this.index}][description]"
            name="description"
            content="${this.data.description}"
            .onChange="${(value) => this._onTextareaChange(value)}"
            mini
            ?required=${!this.isObjectEmpty}
          ></markdown-editor>
        </div>
      </div>

      <div class="col-span-3">
        <label class="form-label"> Start date <span class="asterisk">*</span> </label>
        <div class="mt-2">
          <input
            type="date"
            @input=${(e) => this._onInputChange(e)}
            data-name="start_date"
            name="education[${this.index}][start_date]"
            class="input-primary"
            value="${this.data.start_date || ""}"
            ?required=${!this.isObjectEmpty}
          />
        </div>
      </div>

      <div class="col-span-3">
        <label class="form-label"> End date <span class="asterisk">*</span> </label>
        <div class="mt-2">
          <input
            type="date"
            @input=${(e) => this._onInputChange(e)}
            data-name="end_date"
            name="education[${this.index}][end_date]"
            class="input-primary"
            value="${this.data.end_date || ""}"
            ?required=${!this.isObjectEmpty}
          />
        </div>
      </div>
    </div>`;
  }
}
customElements.define("education-item", EducationItem);
