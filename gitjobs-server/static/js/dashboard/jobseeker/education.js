import { LitElement, html, repeat } from "https://cdn.jsdelivr.net/gh/lit/dist@3/all/lit-all.min.js";
import { isObjectEmpty } from "/static/js/common/common.js";

export class EducationSection extends LitElement {
  static properties = {
    education: { type: Array },
  };

  constructor() {
    super();
    this.education = [];
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
    if (this.education === null) {
      this.education = [this._getData()];
    } else {
      this.education = this.education.map((item, index) => {
        return { ...item, id: index };
      });
    }
  }

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

  _addEducationItem(index) {
    const currentEducation = [...this.education];
    currentEducation.splice(index, 0, this._getData());

    this.education = currentEducation;
  }

  _removeEducationItem(index) {
    const tmpEducation = this.education.filter((_, i) => i !== index);
    // If there are no more education items, add a new one
    this.education = tmpEducation.length === 0 ? [this._getData()] : tmpEducation;
  }

  _onDataChange = (data, index) => {
    this.education[index] = data;
  };

  _getEducationForm(index, education) {
    const hasSingleEducationItem = this.education.length === 1;

    return html`<div class="mt-10">
      <div class="flex w-3/4 lg:w-2/3">
        <div class="flex flex-col space-y-3 me-3">
          <div>
            <button
              @click=${() => this._addEducationItem(index)}
              type="button"
              class="p-2 border hover:bg-gray-100 rounded-full"
              title="Add above"
            >
              <div class="svg-icon size-4 icon-plus_top bg-gray-600"></div>
            </button>
          </div>
          <div>
            <button
              @click=${() => this._addEducationItem(index + 1)}
              type="button"
              class="p-2 border hover:bg-gray-100 rounded-full"
              title="Add below"
            >
              <div class="svg-icon size-4 icon-plus_bottom bg-gray-600"></div>
            </button>
          </div>
          <div>
            <button
              @click=${() => this._removeEducationItem(index)}
              type="button"
              class="p-2 border hover:bg-gray-100 rounded-full"
              title="${hasSingleEducationItem ? "Clean" : "Delete"}"
            >
              <div
                class="svg-icon size-4 icon-${hasSingleEducationItem ? "eraser" : "trash"} bg-gray-600"
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
    return html`<div class="text-xl lg:text-2xl font-medium text-gray-900">Education</div>
      <div class="mt-1 text-sm/6 text-gray-500">
        Indicate your education. You can add additional entries by clicking on the
        <span class="font-semibold">+</span> buttons on the left of the card (
        <div class="inline-block svg-icon size-4 icon-plus_top bg-gray-600 relative -bottom-[2px]"></div>
        to add the new entry above,
        <div class="inline-block svg-icon size-4 icon-plus_bottom bg-gray-600 relative -bottom-[2px]"></div>
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

class EducationItem extends LitElement {
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

  createRenderRoot() {
    // Disable shadow dom to use Tailwind CSS
    return this;
  }

  connectedCallback() {
    super.connectedCallback();
    this.isObjectEmpty = isObjectEmpty(this.data);
  }

  _onInputChange = (e) => {
    const value = e.target.value;
    const name = e.target.dataset.name;

    this.data[name] = value;
    this.isObjectEmpty = isObjectEmpty(this.data);
    this.onDataChange(this.data, this.index);
  };

  _onTextareaChange = (value) => {
    this.data.description = value;
    this.isObjectEmpty = isObjectEmpty(this.data);
    this.onDataChange(this.data, this.index);
  };

  render() {
    return html` <div
      class="grid grid-cols-1 gap-x-6 gap-y-8 sm:grid-cols-6 border border-2 border-dashed p-8 rounded-lg bg-gray-50/25 w-full"
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
