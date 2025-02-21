import { LitElement, html, repeat } from "https://cdn.jsdelivr.net/gh/lit/dist@3/all/lit-all.min.js";

export class CertificationsSection extends LitElement {
  static properties = {
    certifications: { type: Array },
  };

  constructor() {
    super();
    this.certifications = [];
  }

  connectedCallback() {
    super.connectedCallback();
    this.addId();
  }

  createRenderRoot() {
    // Disable shadow dom to use Tailwind CSS
    return this;
  }

  addId() {
    if (this.certifications === null) {
      this.certifications = [];
    } else {
      this.certifications = this.certifications.map((item, index) => {
        return { ...item, id: index };
      });
    }
  }

  _getData = () => {
    let item = {
      id: this.certifications.length,
      title: "",
      provider: "",
      description: "",
      start_date: "",
      end_date: "",
    };

    return item;
  };

  _addCertificationEntry() {
    this.certifications = [...this.certifications, this._getData()];
  }

  _removeCertificationEntry(index) {
    this.certifications = this.certifications.filter((_, i) => i !== index);
  }

  _onInputChange = (e, index) => {
    const value = e.target.value;
    const name = e.target.dataset.name;

    this.certifications[index][name] = value;
  };

  _onTextareaChange = (value, index) => {
    this.certifications[index].description = value;
  };

  _getCertificationEntry(index, certification) {
    let data = this._getData(index, certification);
    if (certification) {
      data = certification;
    }

    return html`<div class="mt-10">
      <div class="flex">
        <div
          class="grid grid-cols-1 gap-x-6 gap-y-8 sm:grid-cols-6 border border-2 border-dashed p-8 rounded-lg bg-gray-50/25 w-3/4 lg:w-2/3"
        >
          <div class="col-span-3">
            <label class="form-label"> Title <span class="asterisk">*</span> </label>
            <div class="mt-2">
              <input
                @input=${(e) => this._onInputChange(e, index)}
                data-name="title"
                type="text"
                name="certifications[${index}][title]"
                class="input-primary"
                value="${data.title || ""}"
                autocomplete="off"
                autocorrect="off"
                autocapitalize="off"
                spellcheck="false"
                required
              />
            </div>
          </div>

          <div class="col-span-3">
            <label class="form-label"> Provider <span class="asterisk">*</span> </label>
            <div class="mt-2">
              <input
                @input=${(e) => this._onInputChange(e, index)}
                data-name="provider"
                type="text"
                name="certifications[${index}][provider]"
                class="input-primary"
                value="${data.provider || ""}"
                autocomplete="off"
                autocorrect="off"
                autocapitalize="off"
                spellcheck="false"
                required
              />
            </div>
          </div>

          <div class="col-span-full">
            <label for="summary" class="form-label"> Description <span class="asterisk">*</span> </label>
            <div class="mt-2">
              <markdown-editor
                id="certifications[${index}][description]"
                name="description"
                content="${data.description || ""}"
                .onChange="${(value) => this._onTextareaChange(value, index)}"
                mini
                required
              ></markdown-editor>
            </div>
          </div>

          <div class="col-span-3">
            <label class="form-label"> Start date <span class="asterisk">*</span> </label>
            <div class="mt-2">
              <input
                type="date"
                @input=${(e) => this._onInputChange(e, index)}
                data-name="start_date"
                name="certifications[${index}][start_date]"
                class="input-primary"
                value="${data.start_date || ""}"
                required
              />
            </div>
          </div>

          <div class="col-span-3">
            <label class="form-label"> End date <span class="asterisk">*</span> </label>
            <div class="mt-2">
              <input
                type="date"
                @input=${(e) => this._onInputChange(e, index)}
                data-name="end_date"
                name="certifications[${index}][end_date]"
                class="input-primary"
                value="${data.end_date || ""}"
                required
              />
            </div>
          </div>
        </div>

        <div class="ms-3">
          <button
            @click=${() => this._removeCertificationEntry(index)}
            type="button"
            class="p-3 hover:bg-gray-100 rounded-full"
          >
            <div class="svg-icon size-4 icon-trash bg-gray-600"></div>
          </button>
        </div>
      </div>
    </div>`;
  }

  render() {
    return html`
      <div class="flex items-center space-x-6">
        <div class="text-xl lg:text-2xl font-medium text-gray-900">Certifications</div>
        <div>
          <button
            @click=${this._addCertificationEntry}
            type="button"
            class="group btn-primary-outline btn-mini"
          >
            <div class="flex items-center space-x-1">
              <div class="svg-icon size-2 icon-plus group-hover:bg-white"></div>
              <div>Add</div>
            </div>
          </button>
        </div>
      </div>
      <p class="mt-1 text-sm/6 text-gray-500">
        Provide certifications you have earned. You can add additional entries by clicking on the
        <span class="italic">Add</span> button next to the title. Entries will be displayed in the order
        provided.
      </p>
      <div id="education-section">
        ${repeat(
          this.certifications,
          (d) => d.id,
          (d, index) => this._getCertificationEntry(index, d),
        )}
      </div>
    `;
  }
}
customElements.define("certifications-section", CertificationsSection);
