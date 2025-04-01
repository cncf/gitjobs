import { html } from "/static/vendor/js/lit-all.v3.2.1.min.js";
import { unnormalize } from "/static/js/common/common.js";
import { triggerActionOnForm } from "/static/js/jobboard/filters.js";
import { LitWrapper } from "/static/js/common/lit-wrapper.js";
import { getBenefits } from "/static/js/common/data.js";

export class SearchableFilter extends LitWrapper {
  static properties = {
    name: { type: String },
    options: { type: Array },
    selected: { type: Array },
    enteredValue: { type: String },
    viewType: { type: String },
    visibleOptions: { type: Array },
    visibleDropdown: { type: Boolean },
    form: { type: String },
    alignment: { type: String },
  };

  constructor() {
    super();
    this.name = "name";
    this.options = [];
    this.selected = [];
    this.enteredValue = "";
    this.viewType = "cols";
    this.visibleOptions = [];
    this.visibleDropdown = false;
    this.form = "";
    this.alignment = "bottom";
  }

  connectedCallback() {
    super.connectedCallback();
    window.addEventListener("mousedown", this.handleClickOutside);
    this._getOptions();
  }

  disconnectedCallback() {
    window.addEventListener("mousedown", this.handleClickOutside);
    super.disconnectedCallback();
  }

  async cleanSelected() {
    this.selected = [];

    // Wait for the update to complete
    await this.updateComplete;
  }

  _getOptions() {
    switch (this.name) {
      case "benefits":
        this.options = getBenefits();
        break;
      case "projects":
        const projects = document.getElementById("projects-list");
        if (projects) {
          this.options = JSON.parse(projects.dataset.projects);
        }
        break;
      default:
        this.options = this.options;
    }

    this._filterOptions();
  }

  _filterOptions() {
    if (this.enteredValue.length > 0) {
      this.visibleOptions = this.options.filter((option) => {
        const name = this.name === "projects" ? option.name : unnormalize(option);
        return name.toLowerCase().includes(this.enteredValue.toLowerCase());
      });
    } else {
      this.visibleOptions = this.options;
    }
  }

  _onInputChange(event) {
    this.enteredValue = event.target.value;
    this._filterOptions();
  }

  _cleanEnteredValue() {
    this.enteredValue = "";
    this.visibleDropdown = false;
    this._filterOptions();
  }

  handleClickOutside = (e) => {
    if (!this.contains(e.target)) {
      this.visibleDropdown = false;
    }
  };

  async _onSelect(value) {
    this.selected.push(value);
    this.enteredValue = "";
    this.visibleDropdown = false;
    this._filterOptions();

    // Wait for the update to complete
    await this.updateComplete;

    // Trigger change event on the form
    triggerActionOnForm(this.form, "submit");
  }

  async _onRemove(value) {
    this.selected = this.selected.filter((item) => item !== value);

    // Wait for the update to complete
    await this.updateComplete;

    // Trigger change event on the form
    triggerActionOnForm(this.form, "submit");
  }

  render() {
    return html`<div class="mt-2 relative">
      <div class="absolute top-2 start-0 flex items-center ps-3 pointer-events-none">
        <div class="svg-icon size-3.5 icon-search bg-stone-300"></div>
      </div>
      <input
        type="text"
        @input=${this._onInputChange}
        @focus=${() => (this.visibleDropdown = true)}
        .value="${this.enteredValue}"
        class="input-primary py-0.5 peer ps-9 rounded-lg text-[0.8rem]/6"
        placeholder="Search ${this.name}"
        autocomplete="off"
        autocorrect="off"
        autocapitalize="off"
        spellcheck="false"
        autocomplete="off"
      />
      <div class="absolute end-1.5 top-0.5 peer-placeholder-shown:hidden">
        <button @click=${this._cleanEnteredValue} type="button" class="cursor-pointer mt-[2px]">
          <div class="svg-icon size-5 bg-stone-400 hover:bg-stone-700 icon-close"></div>
        </button>
      </div>
      <div class="absolute z-10 start-0 end-0 ${this.alignment === "top" ? "-top-[193px] h-[186px]" : ""}">
        <div
          class="${this.alignment === "top" ? "h-full" : ""} ${!this.visibleDropdown
            ? "hidden"
            : ""} bg-white divide-y divide-stone-100 rounded-lg shadow w-full border border-stone-200 mt-1"
        >
          ${this.visibleOptions.length > 0 && this.visibleDropdown
            ? html`<ul class="text-sm text-stone-700 overflow-auto max-h-[180px]">
                ${this.visibleOptions.map((option) => {
                  const isProjectsType = this.name === "projects";
                  const name = isProjectsType ? option.name : option;
                  const isSelected = this.selected.includes(name);
                  return html`<li class="group" data-index="{{ loop.index }}">
                    <button
                      type="button"
                      @click=${() => this._onSelect(name)}
                      class=${`${
                        isSelected ? "bg-stone-100 opacity-50" : "cursor-pointer hover:bg-stone-100"
                      } capitalize block w-full text-left px-${isProjectsType ? "3" : "4"} py-1`}
                      ?disabled="${isSelected}"
                    >
                      ${isProjectsType
                        ? html`<div class="flex items-center space-x-3">
                            <div class="size-8 shrink-0 flex items-center justify-center">
                              <img
                                loading="lazy"
                                class="size-8 object-contain"
                                height="auto"
                                width="auto"
                                src="${option.logo_url}"
                                alt="${name} logo"
                              />
                            </div>
                            <div class="flex flex-col justify-start min-w-0">
                              <div class="truncate text-start text-xs/5 text-stone-700 font-medium">
                                ${name}
                              </div>
                              <div class="inline-flex">
                                <div
                                  class="truncate text-nowrap uppercase max-w-[100%] text-[0.65rem] font-medium text-stone-500/75"
                                >
                                  ${option.foundation} ${option.maturity}
                                </div>
                              </div>
                            </div>
                          </div>`
                        : html`<div class="flex items-center">
                            <div class="size-3 me-2">
                              ${isSelected
                                ? html`<div class="svg-icon size-3 icon-check bg-stone-400"></div>`
                                : ""}
                            </div>
                            <div class="truncate text-[0.8rem]/6">${unnormalize(name)}</div>
                          </div>`}
                    </button>
                  </li>`;
                })}
              </ul>`
            : html`<div class="px-8 py-4 text-sm/6 text-stone-600 italic">No ${this.name} found</div>`}
        </div>
      </div>
      ${this.selected.length > 0
        ? html`<div class="flex gap-2 mt-4 ${this.viewType === "rows" ? "flex-col" : "flex-wrap"}">
            ${this.selected.map(
              (opt) =>
                html` <button
                    type="button"
                    @click=${() => this._onRemove(opt)}
                    class="inline-flex items-center justify-between ps-2 pe-1 py-1 bg-white border rounded-lg cursor-pointer select-none border-primary-500 text-primary-500 max-w-full group"
                  >
                    <div class="flex items-center justify-between space-x-3 w-full">
                      <div class="text-[0.8rem] text-center text-nowrap capitalize truncate">
                        ${unnormalize(opt)}
                      </div>
                      <div
                        class="svg-icon size-4 icon-close bg-stone-500 group-hover:bg-stone-800 shrink-0"
                      ></div>
                    </div>
                  </button>
                  <input type="hidden" form="${this.form}" name="${this.name}[]" value="${opt}" />`,
            )}
          </div>`
        : ""}
    </div>`;
  }
}
customElements.define("searchable-filter", SearchableFilter);
