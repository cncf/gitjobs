import { html, repeat } from "https://cdn.jsdelivr.net/gh/lit/dist@3/all/lit-all.min.js";
import { isObjectEmpty } from "/static/js/common/common.js";
import { LitWrapper } from "/static/js/common/litWrapper.js";

export class ExperienceSection extends LitWrapper {
  static properties = {
    experience: { type: Array },
  };

  constructor() {
    super();
    this.experience = [];
  }

  connectedCallback() {
    super.connectedCallback();
    this._addId();
  }

  _addId() {
    if (this.experience === null) {
      this.experience = [this._getData()];
    } else {
      this.experience = this.experience.map((item, index) => {
        return { ...item, id: index };
      });
    }
  }

  _getData = () => {
    let item = {
      id: this.experience ? this.experience.length : 0,
      title: "",
      company: "",
      description: "",
      start_date: "",
      end_date: "",
    };

    return item;
  };

  _addExperienceRecord(index) {
    const currentExperience = [...this.experience];
    currentExperience.splice(index, 0, this._getData());

    this.experience = currentExperience;
  }

  _removeExperienceRecord(index) {
    const tmpExperience = this.experience.filter((_, i) => i !== index);
    // If there are no more records, add a new one
    this.experience = tmpExperience.length === 0 ? [this._getData()] : tmpExperience;
  }

  _onDataChange = (data, index) => {
    this.experience[index] = data;
  };

  _getExperienceRecord(experience, index) {
    const hasSingleExperienceRecord = this.experience.length === 1;

    return html`<div class="mt-10">
      <div class="flex w-3/4 lg:w-2/3">
        <div class="flex flex-col space-y-3 me-3">
          <div>
            <button
              @click=${() => this._addExperienceRecord(index)}
              type="button"
              class="p-2 border hover:bg-stone-100 rounded-full"
              title="Add above"
            >
              <div class="svg-icon size-4 icon-plus_top bg-stone-600"></div>
            </button>
          </div>
          <div>
            <button
              @click=${() => this._addExperienceRecord(index + 1)}
              type="button"
              class="p-2 border hover:bg-stone-100 rounded-full"
              title="Add below"
            >
              <div class="svg-icon size-4 icon-plus_bottom bg-stone-600"></div>
            </button>
          </div>
          <div>
            <button
              @click=${() => this._removeExperienceRecord(index)}
              type="button"
              class="p-2 border hover:bg-stone-100 rounded-full"
              title="${hasSingleExperienceRecord ? "Clean" : "Delete"}"
            >
              <div
                class="svg-icon size-4 icon-${hasSingleExperienceRecord ? "eraser" : "trash"} bg-stone-600"
              ></div>
            </button>
          </div>
        </div>
        <experience-record
          .data=${experience}
          .index=${index}
          .onDataChange=${this._onDataChange}
          class="w-full"
        ></experience-record>
      </div>
    </div>`;
  }

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
          this.experience,
          (e) => e.id,
          (e, index) => this._getExperienceRecord(e, index),
        )}
      </div>
    `;
  }
}
customElements.define("experience-section", ExperienceSection);

class ExperienceRecord extends LitWrapper {
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
      company: "",
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
    return html`
      <div
        class="grid grid-cols-1 gap-x-6 gap-y-8 sm:grid-cols-6 border border-2 border-dashed p-8 rounded-lg bg-stone-50/25 w-full"
      >
        <div class="col-span-3">
          <label class="form-label"> Title <span class="asterisk">*</span> </label>
          <div class="mt-2">
            <input
              @input=${(e) => this._onInputChange(e)}
              data-name="title"
              type="text"
              name="experience[${this.index}][title]"
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
          <label class="form-label"> Company <span class="asterisk">*</span> </label>
          <div class="mt-2">
            <input
              @input=${(e) => this._onInputChange(e)}
              data-name="company"
              type="text"
              name="experience[${this.index}][company]"
              class="input-primary"
              value="${this.data.company}"
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
              id="experience[${this.index}][description]"
              name="description"
              .content="${this.data.description}"
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
              name="experience[${this.index}][start_date]"
              class="input-primary placeholder-stone-300"
              value="${this.data.start_date}"
              ?required=${!this.isObjectEmpty}
            />
          </div>
        </div>

        <div class="col-span-3">
          <label class="form-label"> End date </label>
          <div class="mt-2">
            <input
              type="date"
              @input=${(e) => this._onInputChange(e)}
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
