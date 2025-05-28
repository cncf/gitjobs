import { html, repeat } from "/static/vendor/js/lit-all.v3.2.1.min.js";
import { isObjectValuesEmpty } from "/static/js/common/common.js";
import { LitWrapper } from "/static/js/common/lit-wrapper.js";

/**
 * ExperienceSection web component for managing a list of professional experience records.
 * Extends LitWrapper for reactive rendering.
 *
 * @property {Array} experienceList - Array of experience record objects.
 */
export class ExperienceSection extends LitWrapper {
  static properties = {
    experienceList: { type: Array },
  };

  /**
   * Initializes the experienceList array.
   */
  constructor() {
    super();
    this.experienceList = [];
  }

  /**
   * Lifecycle method called when the component is added to the DOM.
   * Assigns IDs to experience records.
   */
  connectedCallback() {
    super.connectedCallback();
    this._assignExperienceIds();
  }

  /**
   * Assigns a unique id to each experience record based on its index.
   * If experienceList is null, initializes with a default record.
   */
  _assignExperienceIds() {
    if (this.experienceList === null) {
      this.experienceList = [this._getDefaultExperience()];
    } else {
      this.experienceList = this.experienceList.map((item, index) => {
        return { ...item, id: index };
      });
    }
  }

  /**
   * Returns a default experience record object.
   * @returns {Object} Default experience record.
   */
  _getDefaultExperience = () => {
    return {
      id: this.experienceList ? this.experienceList.length : 0,
      title: "",
      company: "",
      description: "",
      start_date: "",
      end_date: "",
    };
  };

  /**
   * Adds a new experience record at the specified index.
   * @param {number} index - Index to insert the new experience record.
   */
  _addExperience(index) {
    const updatedList = [...this.experienceList];
    updatedList.splice(index, 0, this._getDefaultExperience());
    this.experienceList = updatedList;
  }

  /**
   * Removes an experience record at the specified index.
   * If none remain, adds a default experience record.
   * @param {number} index - Index of the experience record to remove.
   */
  _removeExperience(index) {
    const updatedList = this.experienceList.filter((_, i) => i !== index);
    this.experienceList = updatedList.length === 0 ? [this._getDefaultExperience()] : updatedList;
  }

  /**
   * Handles updates to an experience record.
   * @param {Object} updatedExperience - The updated experience record object.
   * @param {number} index - Index of the experience record to update.
   */
  _handleExperienceChange = (updatedExperience, index) => {
    this.experienceList[index] = updatedExperience;
  };

  /**
   * Renders a single experience record with controls.
   * @param {Object} experience - Experience record data.
   * @param {number} index - Index of the experience record.
   * @returns {TemplateResult} Rendered experience record.
   */
  _renderExperienceRecord(experience, index) {
    const isSingleRecord = this.experienceList.length === 1;
    return html`<div class="mt-10">
      <div class="flex w-full xl:w-2/3">
        <div class="flex flex-col space-y-3 me-3">
          <div>
            <button
              @click=${() => this._addExperience(index)}
              type="button"
              class="cursor-pointer p-2 border border-stone-200 hover:bg-stone-100 rounded-full"
              title="Add above"
            >
              <div class="svg-icon size-4 icon-plus_top bg-stone-600"></div>
            </button>
          </div>
          <div>
            <button
              @click=${() => this._addExperience(index + 1)}
              type="button"
              class="cursor-pointer p-2 border border-stone-200 hover:bg-stone-100 rounded-full"
              title="Add below"
            >
              <div class="svg-icon size-4 icon-plus_bottom bg-stone-600"></div>
            </button>
          </div>
          <div>
            <button
              @click=${() => this._removeExperience(index)}
              type="button"
              class="cursor-pointer p-2 border border-stone-200 hover:bg-stone-100 rounded-full"
              title="${isSingleRecord ? "Clean" : "Delete"}"
            >
              <div class="svg-icon size-4 icon-${isSingleRecord ? "eraser" : "trash"} bg-stone-600"></div>
            </button>
          </div>
        </div>
        <experience-record
          .data=${experience}
          .index=${index}
          .onDataChange=${this._handleExperienceChange}
          class="w-full"
        ></experience-record>
      </div>
    </div>`;
  }

  /**
   * Renders the experience section with all records.
   * @returns {TemplateResult} Rendered section.
   */
  render() {
    return html`
      <div class="text-sm/6 text-stone-500">
        Provide your professional experience. You can add additional entries by clicking on the
        <span class="font-semibold">+</span> buttons on the left of the card (
        <div class="inline-block svg-icon size-4 icon-plus_top bg-stone-600 relative -bottom-[2px]"></div>
        to add the new entry above,
        <div class="inline-block svg-icon size-4 icon-plus_bottom bg-stone-600 relative -bottom-[2px]"></div>
        to add it below). Entries will be displayed in the order provided.
      </div>
      <div id="experience-section">
        ${repeat(
          this.experienceList,
          (e) => e.id,
          (e, index) => this._renderExperienceRecord(e, index),
        )}
      </div>
    `;
  }
}
customElements.define("experience-section", ExperienceSection);

/**
 * ExperienceRecord web component for a single experience entry.
 * Extends LitWrapper for reactive rendering.
 *
 * @property {Object} data - Experience record data object.
 * @property {number} index - Index of this entry in the list.
 * @property {boolean} isObjectValuesEmpty - True if all fields are empty.
 * @property {Function} onDataChange - Callback for data changes.
 */
class ExperienceRecord extends LitWrapper {
  static properties = {
    data: { type: Object },
    index: { type: Number },
    isObjectValuesEmpty: { type: Boolean },
    onDataChange: { type: Function },
  };

  /**
   * Initializes the experience record with default values.
   */
  constructor() {
    super();
    this.data = {
      id: 0,
      title: "",
      company: "",
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
   * Renders the experience record form.
   * @returns {TemplateResult} Rendered entry.
   */
  render() {
    return html`
      <div
        class="grid grid-cols-1 gap-x-6 gap-y-8 sm:grid-cols-6 border-2 border-stone-200 border-dashed p-8 rounded-lg bg-stone-50/25 w-full"
      >
        <div class="col-span-3">
          <label class="form-label"> Title <span class="asterisk">*</span> </label>
          <div class="mt-2">
            <input
              @input=${(e) => this._handleInputChange(e)}
              data-name="title"
              type="text"
              name="experience[${this.index}][title]"
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
          <label class="form-label"> Company <span class="asterisk">*</span> </label>
          <div class="mt-2">
            <input
              @input=${(e) => this._handleInputChange(e)}
              data-name="company"
              type="text"
              name="experience[${this.index}][company]"
              class="input-primary"
              value="${this.data.company}"
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
              id="experience[${this.index}][description]"
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
              name="experience[${this.index}][start_date]"
              class="input-primary placeholder-stone-300"
              value="${this.data.start_date}"
              ?required=${!this.isObjectValuesEmpty}
            />
          </div>
        </div>

        <div class="col-span-3">
          <label class="form-label"> End date </label>
          <div class="mt-2">
            <input
              type="date"
              @input=${(e) => this._handleInputChange(e)}
              data-name="end_date"
              name="experience[${this.index}][end_date]"
              class="input-primary"
              value="${this.data.end_date}"
            />
          </div>
        </div>
      </div>
    `;
  }
}
customElements.define("experience-record", ExperienceRecord);
