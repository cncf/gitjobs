import { html, repeat } from "/static/vendor/js/lit-all.v3.2.1.min.js";
import { isObjectValuesEmpty } from "/static/js/common/common.js";
import { LitWrapper } from "/static/js/common/lit-wrapper.js";

/**
 * EducationSection web component for managing a list of education entries.
 * Extends LitWrapper for reactive rendering.
 *
 * @property {Array} educationList - Array of education entry objects.
 */
export class EducationSection extends LitWrapper {
  static properties = {
    educationList: { type: Array },
  };

  /**
   * Initializes the educationList array.
   */
  constructor() {
    super();
    this.educationList = [];
  }

  /**
   * Lifecycle method called when the component is added to the DOM.
   * Assigns IDs to education entries.
   */
  connectedCallback() {
    super.connectedCallback();
    this._assignEducationIds();
  }

  /**
   * Assigns a unique id to each education entry based on its index.
   * If educationList is null, initializes with a default entry.
   */
  _assignEducationIds() {
    if (this.educationList === null) {
      this.educationList = [this._getDefaultEducationEntry()];
    } else {
      this.educationList = this.educationList.map((entry, index) => {
        return { ...entry, id: index };
      });
    }
  }

  /**
   * Returns a default education entry object.
   * @returns {Object} Default education entry.
   */
  _getDefaultEducationEntry = () => {
    return {
      id: this.educationList ? this.educationList.length : 0,
      title: "",
      educational_institution: "",
      description: "",
      start_date: "",
      end_date: "",
    };
  };

  /**
   * Adds a new education entry at the specified index.
   * @param {number} index - Index to insert the new education entry.
   */
  _addEducationEntry(index) {
    const updatedList = [...this.educationList];
    updatedList.splice(index, 0, this._getDefaultEducationEntry());
    this.educationList = updatedList;
  }

  /**
   * Removes an education entry at the specified index.
   * If none remain, adds a default education entry.
   * @param {number} index - Index of the education entry to remove.
   */
  _removeEducationEntry(index) {
    const updatedList = this.educationList.filter((_, i) => i !== index);
    this.educationList = updatedList.length === 0 ? [this._getDefaultEducationEntry()] : updatedList;
  }

  /**
   * Handles updates to an education entry.
   * @param {Object} updatedEducation - The updated education entry object.
   * @param {number} index - Index of the education entry to update.
   */
  _handleEducationChange = (updatedEducation, index) => {
    this.educationList[index] = updatedEducation;
  };

  /**
   * Renders a single education entry with controls.
   * @param {number} index - Index of the education entry.
   * @param {Object} education - Education entry data.
   * @returns {TemplateResult} Rendered education entry.
   */
  _renderEducationEntry(index, education) {
    const isSingleEntry = this.educationList.length === 1;
    return html`<div class="mt-10">
      <div class="flex w-full xl:w-2/3">
        <div class="flex flex-col space-y-3 me-3">
          <div>
            <button
              @click=${() => this._addEducationEntry(index)}
              type="button"
              class="cursor-pointer p-2 border border-stone-200 hover:bg-stone-100 rounded-full"
              title="Add above"
            >
              <div class="svg-icon size-4 icon-plus_top bg-stone-600"></div>
            </button>
          </div>
          <div>
            <button
              @click=${() => this._addEducationEntry(index + 1)}
              type="button"
              class="cursor-pointer p-2 border border-stone-200 hover:bg-stone-100 rounded-full"
              title="Add below"
            >
              <div class="svg-icon size-4 icon-plus_bottom bg-stone-600"></div>
            </button>
          </div>
          <div>
            <button
              @click=${() => this._removeEducationEntry(index)}
              type="button"
              class="cursor-pointer p-2 border border-stone-200 hover:bg-stone-100 rounded-full"
              title="${isSingleEntry ? "Clean" : "Delete"}"
            >
              <div class="svg-icon size-4 icon-${isSingleEntry ? "eraser" : "trash"} bg-stone-600"></div>
            </button>
          </div>
        </div>
        <education-entry
          .data=${education}
          .index=${index}
          .onDataChange=${this._handleEducationChange}
          class="w-full"
        ></education-entry>
      </div>
    </div>`;
  }

  /**
   * Renders the education section with all entries.
   * @returns {TemplateResult} Rendered section.
   */
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
          this.educationList,
          (e) => e.id,
          (e, index) => this._renderEducationEntry(index, e),
        )}
      </div>`;
  }
}
customElements.define("education-section", EducationSection);

/**
 * EducationEntry web component for a single education entry.
 * Extends LitWrapper for reactive rendering.
 *
 * @property {Object} data - Education entry data object.
 * @property {number} index - Index of this entry in the list.
 * @property {boolean} isObjectValuesEmpty - True if all fields are empty.
 * @property {Function} onDataChange - Callback for data changes.
 */
class EducationEntry extends LitWrapper {
  static properties = {
    data: { type: Object },
    index: { type: Number },
    isObjectValuesEmpty: { type: Boolean },
    onDataChange: { type: Function },
  };

  /**
   * Initializes the education entry with default values.
   */
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
    this.isObjectValuesEmpty = true;
    this.onDataChange = () => {};
  }

  /**
   * Lifecycle method called when the component is added to the DOM.
   * Sets the isObjectValuesEmpty property based on data.
   */
  connectedCallback() {
    super.connectedCallback();
    this.isObjectValuesEmpty = isObjectValuesEmpty(this.data);
  }

  /**
   * Handles input changes for text and date fields.
   * Updates the data object and notifies parent.
   * @param {Event} e - Input event.
   */
  _handleInputChange = (e) => {
    const value = e.target.value;
    const name = e.target.dataset.name;
    this.data[name] = value;
    this.isObjectValuesEmpty = isObjectValuesEmpty(this.data);
    this.onDataChange(this.data, this.index);
  };

  /**
   * Handles changes to the description field.
   * @param {string} value - New description value.
   */
  _handleDescriptionChange = (value) => {
    this.data.description = value;
    this.isObjectValuesEmpty = isObjectValuesEmpty(this.data);
    this.onDataChange(this.data, this.index);
  };

  /**
   * Renders the education entry form.
   * @returns {TemplateResult} Rendered entry.
   */
  render() {
    return html` <div
      class="grid grid-cols-1 gap-x-6 gap-y-8 sm:grid-cols-6 border-2 border-stone-200 border-dashed p-8 rounded-lg bg-stone-50/25 w-full"
    >
      <div class="col-span-3">
        <label class="form-label"> Title <span class="asterisk">*</span> </label>
        <div class="mt-2">
          <input
            @input=${(e) => this._handleInputChange(e)}
            data-name="title"
            type="text"
            name="education[${this.index}][title]"
            class="input-primary"
            value="${this.data.title}"
            autocomplete="off"
            autocorrect="off"
            autocapitalize="off"
            spellcheck="false"
            ?required=${!this.isObjectValuesEmpty}
          />
        </div>
      </div>

      <div class="col-span-3">
        <label class="form-label"> Educational institution <span class="asterisk">*</span> </label>
        <div class="mt-2">
          <input
            @input=${(e) => this._handleInputChange(e)}
            data-name="educational_institution"
            type="text"
            name="education[${this.index}][educational_institution]"
            class="input-primary"
            value="${this.data.educational_institution}"
            autocomplete="off"
            autocorrect="off"
            autocapitalize="off"
            spellcheck="false"
            ?required=${!this.isObjectValuesEmpty}
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
            .onContentChange="${(value) => this._handleDescriptionChange(value)}"
            mini
            ?required=${!this.isObjectValuesEmpty}
          ></markdown-editor>
        </div>
      </div>

      <div class="col-span-3">
        <label class="form-label"> Start date <span class="asterisk">*</span> </label>
        <div class="mt-2">
          <input
            type="date"
            @input=${(e) => this._handleInputChange(e)}
            data-name="start_date"
            name="education[${this.index}][start_date]"
            class="input-primary"
            value="${this.data.start_date || ""}"
            ?required=${!this.isObjectValuesEmpty}
          />
        </div>
      </div>

      <div class="col-span-3">
        <label class="form-label"> End date <span class="asterisk">*</span> </label>
        <div class="mt-2">
          <input
            type="date"
            @input=${(e) => this._handleInputChange(e)}
            data-name="end_date"
            name="education[${this.index}][end_date]"
            class="input-primary"
            value="${this.data.end_date || ""}"
            ?required=${!this.isObjectValuesEmpty}
          />
        </div>
      </div>
    </div>`;
  }
}
customElements.define("education-entry", EducationEntry);
