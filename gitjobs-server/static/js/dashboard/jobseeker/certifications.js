import { html, repeat } from "/static/vendor/js/lit-all.v3.2.1.min.js";
import { isObjectEmpty } from "/static/js/common/common.js";
import { LitWrapper } from "/static/js/common/lit-wrapper.js";

/**
 * Component for managing certification entries in job seeker profile.
 * Supports adding, removing, and reordering certifications.
 * @extends LitWrapper
 */
export class CertificationsSection extends LitWrapper {
  /**
   * Component properties definition
   * @property {Array} certifications - List of certification entries
   * Each entry contains:
   *   - id: Unique identifier
   *   - title: Certification title
   *   - provider: Issuing organization
   *   - description: Certification details
   *   - start_date: Date when certification was obtained
   *   - end_date: Date when certification expires (if applicable)
   */
  static properties = {
    certifications: { type: Array },
  };

  constructor() {
    super();
    this.certifications = [];
  }

  connectedCallback() {
    super.connectedCallback();
    this._initializeCertificationIds();
  }

  /**
   * Assigns unique IDs to certification entries.
   * Creates initial entry if none exist.
   * @private
   */
  _initializeCertificationIds() {
    if (this.certifications === null) {
      this.certifications = [this._getData()];
    } else {
      this.certifications = this.certifications.map((item, index) => {
        return { ...item, id: index };
      });
    }
  }

  /**
   * Creates a new empty certification data object.
   * @returns {Object} Empty certification entry
   * @private
   */
  _getData = () => {
    let item = {
      id: this.certifications ? this.certifications.length : 0,
      title: "",
      provider: "",
      description: "",
      start_date: "",
      end_date: "",
    };

    return item;
  };

  /**
   * Adds a new certification entry at specified index.
   * @param {number} index - Position to insert new entry
   * @private
   */
  _addCertificationEntry(index) {
    const currentCertifications = [...this.certifications];
    currentCertifications.splice(index, 0, this._getData());

    this.certifications = currentCertifications;
  }

  /**
   * Removes certification entry at specified index.
   * Ensures at least one empty entry remains.
   * @param {number} index - Position of entry to remove
   * @private
   */
  _removeCertificationEntry(index) {
    const tmpCertifications = this.certifications.filter((_, i) => i !== index);

    // If there are no more certifications, add a new one
    this.certifications = tmpCertifications.length === 0 ? [this._getData()] : tmpCertifications;
  }

  /**
   * Updates certification data at specified index.
   * @param {Object} data - Updated certification data
   * @param {number} index - Index of entry to update
   * @private
   */
  _onDataChange = (data, index) => {
    this.certifications[index] = data;
  };

  /**
   * Renders a certification entry with controls.
   * @param {number} index - Entry index
   * @param {Object} certification - Certification data
   * @returns {import('lit').TemplateResult} Entry template
   * @private
   */
  _getCertificationEntry(index, certification) {
    const hasSingleCertificationEntry = this.certifications.length === 1;

    return html`<div class="mt-10">
      <div class="flex w-full xl:w-2/3">
        <div class="flex flex-col space-y-3 me-3">
          <div>
            <button
              @click=${() => this._addCertificationEntry(index)}
              type="button"
              class="cursor-pointer p-2 border border-stone-200 hover:bg-stone-100 rounded-full"
              title="Add above"
            >
              <div class="svg-icon size-4 icon-plus_top bg-stone-600"></div>
            </button>
          </div>
          <div>
            <button
              @click=${() => this._addCertificationEntry(index + 1)}
              type="button"
              class="cursor-pointer p-2 border border-stone-200 hover:bg-stone-100 rounded-full"
              title="Add below"
            >
              <div class="svg-icon size-4 icon-plus_bottom bg-stone-600"></div>
            </button>
          </div>
          <div>
            <button
              @click=${() => this._removeCertificationEntry(index)}
              type="button"
              class="cursor-pointer p-2 border border-stone-200 hover:bg-stone-100 rounded-full"
              title="${hasSingleCertificationEntry ? "Clean" : "Delete"}"
            >
              <div
                class="svg-icon size-4 icon-${hasSingleCertificationEntry ? "eraser" : "trash"} bg-stone-600"
              ></div>
            </button>
          </div>
        </div>
        <certification-entry
          .data=${certification}
          .index=${index}
          .onDataChange=${this._onDataChange}
          class="w-full"
        ></certification-entry>
      </div>
    </div>`;
  }

  render() {
    return html`<div class="text-xl lg:text-2xl font-medium text-stone-900">Certifications</div>
      <div class="mt-2 text-sm/6 text-stone-500">
        Provide certifications you have earned. You can add additional entries by clicking on the
        <span class="font-semibold">+</span> buttons on the left of the card (
        <div class="inline-block svg-icon size-4 icon-plus_top bg-stone-600 relative -bottom-[2px]"></div>
        to add the new entry above,
        <div class="inline-block svg-icon size-4 icon-plus_bottom bg-stone-600 relative -bottom-[2px]"></div>
        to add it below). Entries will be displayed in the order provided.
      </div>
      <div id="certifications-section">
        ${repeat(
          this.certifications,
          (c) => c.id,
          (c, index) => this._getCertificationEntry(index, c),
        )}
      </div> `;
  }
}
customElements.define("certifications-section", CertificationsSection);

/**
 * Individual certification entry component.
 * Handles form inputs and validation for a single certification.
 * @extends LitWrapper
 */
class CertificationEntry extends LitWrapper {
  /**
   * Component properties definition
   * @property {Object} data - Certification data object
   * Contains:
   *   - id: Unique identifier
   *   - title: Certification title
   *   - provider: Issuing organization
   *   - description: Certification details
   *   - start_date: Date when certification was obtained
   *   - end_date: Date when certification expires (if applicable)
   * @property {number} index - Index of the certification entry in the list
   * @property {boolean} isObjectEmpty - Indicates if the certification data is empty
   * @property {Function} onDataChange - Callback function to notify parent component of data changes
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
      provider: "",
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
    return html`<div
      class="grid grid-cols-1 gap-x-6 gap-y-8 sm:grid-cols-6 border-2 border-stone-200 border-dashed p-8 rounded-lg bg-stone-50/25 w-full"
    >
      <div class="col-span-3">
        <label class="form-label"> Title <span class="asterisk">*</span> </label>
        <div class="mt-2">
          <input
            @input=${this._onInputChange}
            data-name="title"
            type="text"
            name="certifications[${this.index}][title]"
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
        <label class="form-label"> Provider <span class="asterisk">*</span> </label>
        <div class="mt-2">
          <input
            @input=${this._onInputChange}
            data-name="provider"
            type="text"
            name="certifications[${this.index}][provider]"
            class="input-primary"
            value="${this.data.provider}"
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
            id="certifications[${this.index}][description]"
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
            @input=${this._onInputChange}
            data-name="start_date"
            name="certifications[${this.index}][start_date]"
            class="input-primary"
            value="${this.data.start_date}"
            ?required=${!this.isObjectEmpty}
          />
        </div>
      </div>

      <div class="col-span-3">
        <label class="form-label"> End date <span class="asterisk">*</span> </label>
        <div class="mt-2">
          <input
            type="date"
            @input=${this._onInputChange}
            data-name="end_date"
            name="certifications[${this.index}][end_date]"
            class="input-primary"
            value="${this.data.end_date}"
            ?required=${!this.isObjectEmpty}
          />
        </div>
      </div>
    </div>`;
  }
}
customElements.define("certification-entry", CertificationEntry);
