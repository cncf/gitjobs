import { LitElement, html } from "https://cdn.jsdelivr.net/gh/lit/dist@3/all/lit-all.min.js";
import { unnormalize } from "/static/js/common/common.js";
import { triggerChangeOnForm } from "/static/js/jobboard/filters.js";

export class SearchableFilter extends LitElement {
  static properties = {
    name: { type: String },
    options: { type: Array },
    selected: { type: Array },
    enteredValue: { type: String },
    viewType: { type: String },
    visibleOptions: { type: Array },
    visibleDropdown: { type: Boolean },
    form: { type: String },
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
  }

  createRenderRoot() {
    if (this.children.length === 0) {
      // Disable shadow dom to use Tailwind CSS
      return this;
    } else {
      // Remove previous content when re-rendering full component
      this.innerHTML = "";
      // Disable shadow dom to use Tailwind CSS
      return this;
    }
  }

  connectedCallback() {
    super.connectedCallback();
    window.addEventListener("mousedown", this.handleClickOutside);
    this.filterOptions();
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

  filterOptions() {
    if (this.enteredValue.length > 0) {
      this.visibleOptions = this.options.filter((option) => {
        const name = this.name === "projects" ? option.name : option;
        return name.toLowerCase().includes(this.enteredValue.toLowerCase());
      });
    } else {
      this.visibleOptions = this.options;
    }
  }

  _onInputChange(event) {
    this.enteredValue = event.target.value;
    this.filterOptions();
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
    this.filterOptions();

    // Wait for the update to complete
    await this.updateComplete;

    // Trigger change event on the form
    triggerChangeOnForm(this.form);
  }

  async _onRemove(value) {
    this.selected = this.selected.filter((item) => item !== value);

    // Wait for the update to complete
    await this.updateComplete;

    // Trigger change event on the form
    triggerChangeOnForm(this.form);
  }

  render() {
    return html`<div class="mt-2 relative">
      <div class="absolute top-2.5 start-0 flex items-center ps-3 pointer-events-none">
        <div class="svg-icon size-4 icon-search bg-gray-300"></div>
      </div>
      <input
        type="text"
        @input=${this._onInputChange}
        @focus=${() => (this.visibleDropdown = true)}
        .value="${this.enteredValue}"
        class="input-primary peer ps-9 rounded-lg"
        placeholder="Search ${this.name}"
        autocomplete="off"
        autocorrect="off"
        autocapitalize="off"
        spellcheck="false"
        autocomplete="off"
      />
      <div class="absolute end-1.5 top-1.5 peer-placeholder-shown:hidden">
        <button type="button" class="mt-[2px]">
          <div class="svg-icon size-5 bg-gray-400 hover:bg-gray-700 icon-close"></div>
        </button>
      </div>
      <div class="absolute z-10 start-0 end-0">
        <div
          class="${!this.visibleDropdown
            ? "hidden"
            : ""} bg-white divide-y divide-gray-100 rounded-lg shadow w-full border mt-1"
        >
          ${this.visibleOptions.length > 0
            ? html`<ul class="py-2 text-sm text-gray-700 overflow-auto max-h-[180px]">
                ${this.visibleOptions.map((option) => {
                  const isProjectsType = this.name === "projects";
                  const name = isProjectsType ? option.name : option;
                  const isSelected = this.selected.includes(name);
                  return html`<li class="group" data-index="{{ loop.index }}">
                    <button
                      type="button"
                      @click=${() => this._onSelect(name)}
                      class=${`${
                        isSelected ? "bg-gray-100 opacity-50" : "cursor-pointer hover:bg-gray-100"
                      } capitalize block w-full text-left px-${isProjectsType ? "3" : "4"} py-2`}
                      ?disabled="${isSelected}"
                    >
                      ${isProjectsType
                        ? html`<div class="flex items-center space-x-3">
                            <img
                              class="size-10"
                              height="40"
                              width="40"
                              src="${option.logo_url}"
                              alt="${name} logo"
                            />
                            <div class="flex flex-col justify-start min-w-0">
                              <div class="truncate text-start text-gray-700 font-medium">${name}</div>
                              <div class="inline-flex">
                                <div
                                  class="truncate text-nowrap uppercase max-w-[100%] text-xs/6 font-medium text-gray-400"
                                >
                                  CNCF ${option.maturity}
                                </div>
                              </div>
                            </div>
                          </div>`
                        : html`<div class="flex items-center">
                            <div class="size-3 me-2">
                              ${isSelected
                                ? html`<div class="svg-icon size-3 icon-check bg-gray-400"></div>`
                                : ""}
                            </div>
                            <div class="truncate">${unnormalize(name)}</div>
                          </div>`}
                    </button>
                  </li>`;
                })}
              </ul>`
            : html`<div class="px-8 py-4 text-sm/6 text-gray-600 italic">No ${this.name} found</div>`}
        </div>
      </div>
      ${this.selected.length > 0
        ? html`<div class="flex gap-2 mt-2 ${this.viewType === "rows" ? "flex-col" : "flex-wrap"}">
            ${this.selected.map(
              (opt) => html` <button
                  type="button"
                  @click=${() => this._onRemove(opt)}
                  class="inline-flex items-center justify-between px-2 py-1 bg-white border rounded-lg cursor-pointer select-none border-primary-500 text-primary-500 max-w-full group"
                >
                  <div class="flex items-center justify-between space-x-3 w-full">
                    <div class="text-[0.8rem] text-center text-nowrap capitalize truncate">
                      ${unnormalize(opt)}
                    </div>
                    <div
                      class="svg-icon size-3 icon-close bg-gray-500 group-hover:bg-gray-800 shrink-0"
                    ></div>
                  </div>
                </button>
                <input type="hidden" name="${this.name}[]" value="${opt}" />`,
            )}
          </div>`
        : ""}
    </div>`;
  }
}
customElements.define("searchable-filter", SearchableFilter);
