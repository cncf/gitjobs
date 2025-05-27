import { html, repeat } from "/static/vendor/js/lit-all.v3.2.1.min.js";
import { isObjectValuesEmpty } from "/static/js/common/common.js";
import { LitWrapper } from "/static/js/common/lit-wrapper.js";

/**
 * CertificationsSection web component for managing a list of certification entries.
 * Extension LitWrapper for reactive rendering.
 *
 * @class CertificationsSection
 * @property {Array} certifications - Array of certification objects.
 */
export class CertificationsSection extends LitWrapper {
  static properties = {
    certifications: { type: Array },
  };

  /**
   * Initializes the certifications array.
   */
  constructor() {
    super();
    this.certifications = [];
  }

  /**
   * Lifecycle method called when the component is added to the DOM.
   * Assigns IDs to certifications.
   */
  connectedCallback() {
    super.connectedCallback();
    this._assignCertificationIds();
  }

  /**
   * Assigns a unique id to each certification based on its index.
   * If certifications is null, initializes with a default certification.
   */
  _assignCertificationIds() {
    if (this.certifications === null) {
      this.certifications = [this._getDefaultCertification()];
    } else {
      this.certifications = this.certifications.map((cert, idx) => {
        return { ...cert, id: idx };
      });
    }
  }

  /**
   * Returns a default certification object.
   * @returns {Object} Default certification.
   */
  _getDefaultCertification = () => {
    return {
      id: this.certifications ? this.certifications.length : 0,
      title: "",
      provider: "",
      description: "",
      start_date: "",
      end_date: "",
    };
  };

  /**
   * Adds a new certification entry at the specified index.
   * @param {number} index - Index to insert the new certification.
   */
  _addCertification = (index) => {
    const certificationsCopy = [...this.certifications];
    certificationsCopy.splice(index, 0, this._getDefaultCertification());
    this.certifications = certificationsCopy;
  };

  /**
   * Removes a certification entry at the specified index.
   * If none remain, adds a default certification.
   * @param {number} index - Index of the certification to remove.
   */
  _removeCertification = (index) => {
    const certificationsCopy = this.certifications.filter((_, i) => i !== index);
    this.certifications =
      certificationsCopy.length === 0 ? [this._getDefaultCertification()] : certificationsCopy;
  };

  /**
   * Handles updates to a certification entry.
   * @param {Object} updatedCertification - The updated certification object.
   * @param {number} index - Index of the certification to update.
   */
  _handleCertificationChange = (updatedCertification, index) => {
    this.certifications[index] = updatedCertification;
  };

  /**
   * Renders a single certification entry with controls.
   * @param {number} index - Index of the certification.
   * @param {Object} certification - Certification data.
   * @returns {TemplateResult} Rendered certification entry.
   */
  _renderCertificationEntry(index, certification) {
    const isSingleEntry = this.certifications.length === 1;
    return html`<div class="mt-10">
      <div class="flex w-full xl:w-2/3">
        <div class="flex flex-col space-y-3 me-3">
          <div>
            <button
              @click=${() => this._addCertification(index)}
              type="button"
              class="cursor-pointer p-2 border border-stone-200 hover:bg-stone-100 rounded-full"
              title="Add above"
            >
              <div class="svg-icon size-4 icon-plus_top bg-stone-600"></div>
            </button>
          </div>
          <div>
            <button
              @click=${() => this._addCertification(index + 1)}
              type="button"
              class="cursor-pointer p-2 border border-stone-200 hover:bg-stone-100 rounded-full"
              title="Add below"
            >
              <div class="svg-icon size-4 icon-plus_bottom bg-stone-600"></div>
            </button>
          </div>
          <div>
            <button
              @click=${() => this._removeCertification(index)}
              type="button"
              class="cursor-pointer p-2 border border-stone-200 hover:bg-stone-100 rounded-full"
              title="${isSingleEntry ? "Clean" : "Delete"}"
            >
              <div class="svg-icon size-4 icon-${isSingleEntry ? "eraser" : "trash"} bg-stone-600"></div>
            </button>
          </div>
        </div>
        <certification-entry
          .data=${certification}
          .index=${index}
          .onDataChange=${this._handleCertificationChange}
          class="w-full"
        ></certification-entry>
      </div>
    </div>`;
  }

  /**
   * Renders the certifications section with all entries.
   * @returns {TemplateResult} Rendered section.
   */
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
          (cert) => cert.id,
          (cert, idx) => this._renderCertificationEntry(idx, cert),
        )}
      </div> `;
  }
}
customElements.define("certifications-section", CertificationsSection);

/**
 * CertificationEntry web component for a single certification form entry.
 * Extension LitWrapper for reactive rendering.
 *
 * @class CertificationEntry
 *  @property {Object} data - Certification data object.
 * @property {number} index - Index of this entry in the list.
 * @property {boolean} isEmpty - True if all fields are empty.
 * @property {Function} onDataChange - Callback for data changes.
 */
class CertificationEntry extends LitWrapper {
  static properties = {
    data: { type: Object },
    index: { type: Number },
    isEmpty: { type: Boolean },
    onDataChange: { type: Function },
  };

  /**
   * Initializes the certification entry with default values.
   */
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
    this.isEmpty = true;
    this.onDataChange = () => {};
  }

  /**
   * Lifecycle method called when the component is added to the DOM.
   * Sets the isEmpty property based on data.
   */
  connectedCallback() {
    super.connectedCallback();
    this.isEmpty = isObjectValuesEmpty(this.data);
  }

  /**
   * Handles input changes for text and date fields.
   * Updates the data object and notifies parent.
   * @param {Event} e - Input event.
   */
  _handleInputChange = (e) => {
    const value = e.target.value;
    const field = e.target.dataset.name;
    this.data[field] = value;
    this.isEmpty = isObjectValuesEmpty(this.data);
    this.onDataChange(this.data, this.index);
  };

  /**
   * Handles changes to the description field.
   * @param {string} value - New description value.
   */
  _handleDescriptionChange = (value) => {
    this.data.description = value;
    this.isEmpty = isObjectValuesEmpty(this.data);
    this.onDataChange(this.data, this.index);
  };

  /**
   * Renders the certification entry form.
   * @returns {TemplateResult} Rendered entry.
   */
  render() {
    return html`<div
      class="grid grid-cols-1 gap-x-6 gap-y-8 sm:grid-cols-6 border-2 border-stone-200 border-dashed p-8 rounded-lg bg-stone-50/25 w-full"
    >
      <div class="col-span-3">
        <label class="form-label"> Title <span class="asterisk">*</span> </label>
        <div class="mt-2">
          <input
            @input=${(e) => this._handleInputChange(e)}
            data-name="title"
            type="text"
            name="certifications[${this.index}][title]"
            class="input-primary"
            value="${this.data.title}"
            autocomplete="off"
            autocorrect="off"
            autocapitalize="off"
            spellcheck="false"
            ?required=${!this.isEmpty}
          />
        </div>
      </div>

      <div class="col-span-3">
        <label class="form-label"> Provider <span class="asterisk">*</span> </label>
        <div class="mt-2">
          <input
            @input=${(e) => this._handleInputChange(e)}
            data-name="provider"
            type="text"
            name="certifications[${this.index}][provider]"
            class="input-primary"
            value="${this.data.provider}"
            autocomplete="off"
            autocorrect="off"
            autocapitalize="off"
            spellcheck="false"
            ?required=${!this.isEmpty}
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
            .onContentChange="${(value) => this._handleDescriptionChange(value)}"
            mini
            ?required=${!this.isEmpty}
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
            name="certifications[${this.index}][start_date]"
            class="input-primary"
            value="${this.data.start_date}"
            ?required=${!this.isEmpty}
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
            name="certifications[${this.index}][end_date]"
            class="input-primary"
            value="${this.data.end_date}"
            ?required=${!this.isEmpty}
          />
        </div>
      </div>
    </div>`;
  }
}
customElements.define("certification-entry", CertificationEntry);
