import { html } from "/static/vendor/js/lit-all.v3.2.1.min.js";
import { unnormalize } from "/static/js/common/common.js";
import { LitWrapper } from "/static/js/common/lit-wrapper.js";
import { getBenefits, getSkills } from "/static/js/common/data.js";

/**
 * Multi-select component with search and badge display.
 * Supports predefined options (benefits, skills) or custom options.
 * @extends LitWrapper
 */
export class MultiSelect extends LitWrapper {
  /**
   * Component properties definition
   * @property {string} name - Input name for form submission
   * @property {string} label - Field label text
   * @property {string[]} options - Available options to select
   * @property {string[]} selected - Currently selected options
   * @property {string} enteredValue - Current search input value
   * @property {string[]} visibleOptions - Filtered options based on search
   * @property {boolean} visibleDropdown - Dropdown visibility state
   * @property {string} legend - Helper text below input
   */
  static properties = {
    name: { type: String },
    label: { type: String },
    options: { type: Array },
    selected: { type: Array },
    enteredValue: { type: String },
    visibleOptions: { type: Array },
    visibleDropdown: { type: Boolean },
    legend: { type: String },
  };

  constructor() {
    super();
    this.name = "name";
    this.label = "label";
    this.options = [];
    this.selected = [];
    this.enteredValue = "";
    this.visibleOptions = [];
    this.visibleDropdown = false;
    this.legend = undefined;
  }

  connectedCallback() {
    super.connectedCallback();
    window.addEventListener("mousedown", this._handleClickOutside);
    this._getOptions();
  }

  disconnectedCallback() {
    super.disconnectedCallback();
    window.addEventListener("mousedown", this._handleClickOutside);
  }

  /**
   * Filters visible options based on search input.
   * @private
   */
  _filterOptions() {
    if (this.enteredValue.length > 0) {
      this.visibleOptions = this.options.filter((option) =>
        unnormalize(option).toLowerCase().includes(this.enteredValue.toLowerCase()),
      );
    } else {
      this.visibleOptions = this.options;
    }
  }

  /**
   * Loads options based on component name (benefits/skills).
   * @private
   */
  _getOptions() {
    switch (this.name) {
      case "benefits":
        this.options = getBenefits();
        break;
      case "skills":
        this.options = getSkills();
        break;
      default:
        this.options = this.options;
    }

    this._filterOptions();
  }

  /**
   * Handles click outside to close dropdown.
   * @param {MouseEvent} event - The click event
   * @private
   */
  _handleClickOutside = (event) => {
    if (!this.contains(event.target)) {
      this.visibleDropdown = false;
    }
  };

  /**
   * Handles search input changes.
   * @param {Event} event - Input event
   * @private
   */
  _onInputChange(event) {
    this.enteredValue = event.target.value;
    this._filterOptions();
  }

  /**
   * Removes a selected option.
   * @param {string} option - Option to remove
   * @private
   */
  _onRemoveBadge(option) {
    this.selected = this.selected.filter((selectedOption) => selectedOption !== option);
  }

  /**
   * Adds an option to selected list.
   * @param {string} option - Option to add, or uses entered value if empty
   * @private
   */
  _onClickOption(option) {
    this.selected.push(option || this.enteredValue);
    this.enteredValue = "";
    this.visibleDropdown = false;
    this._filterOptions();
  }

  render() {
    return html`
      <label for="${this.name}" class="form-label">${this.label}</label>
      <div class="mt-2 relative">
        <div
          class="input-primary px-1.5 flex flex-wrap focus-within:outline-[3px] focus-within:-outline-offset-2 focus-within:outline-primary-600"
        >
          <div class="flex flex-items flex-wrap w-full gap-2">
            ${this.selected.map(
              (option) =>
                html`<span
                  class="inline-flex items-center text-nowrap max-w-[100%] ps-2 pe-0.5 py-0.5 text-xs font-medium text-stone-800 bg-stone-100 rounded-full"
                >
                  <div class="flex items-center w-full">
                    <div class="truncate uppercase">${unnormalize(option)}</div>
                    <button
                      type="button"
                      @click=${() => this._onRemoveBadge(option)}
                      class="inline-flex items-center cursor-pointer p-1 ms-2 bg-transparent rounded-full hover:bg-stone-200"
                      aria-label="Remove badge"
                    >
                      <div class="svg-icon size-3 icon-close bg-stone-400 hover:bg-stone-900"></div>
                      <span class="sr-only">Remove badge</span>
                    </button>
                  </div>
                </span> `,
            )}
            <input
              type="text"
              @input=${this._onInputChange}
              @focus=${() => (this.visibleDropdown = true)}
              .value="${this.enteredValue}"
              placeholder="Type to search"
              autocomplete="off"
              autocorrect="off"
              autocapitalize="off"
              spellcheck="false"
              class="flex grow p-0 ps-1.5 rounded-md text-stone-900 max-w-full min-w-[80px] border-0 focus:ring-0 sm:text-sm/6"
            />
          </div>
        </div>
        ${this.legend ? html`<p class="form-legend">${this.legend}</p>` : ""}
        <div
          class=${`${
            !this.visibleDropdown ? "hidden" : ""
          } absolute start-0 z-10 bg-white divide-y divide-stone-100 rounded-lg shadow w-full border border-stone-200 mt-1 ${
            this.legend ? "top-10" : ""
          }`}
        >
          <ul class="text-sm text-stone-700 overflow-x-auto max-h-[150px]">
            ${this.visibleOptions.map((option) => {
              const isSelected = this.selected.includes(option);
              return html`<li>
                <button
                  @click=${() => this._onClickOption(option)}
                  type="button"
                  class=${`${
                    isSelected ? "bg-stone-100 opacity-50" : "cursor-pointer hover:bg-stone-100"
                  } capitalize block w-full text-left px-4 py-2`}
                  ?disabled="${isSelected}"
                >
                  <div class="flex items-center">
                    <div class="size-3 me-2">
                      ${isSelected ? html`<div class="svg-icon size-3 icon-check bg-stone-400"></div>` : ""}
                    </div>
                    <div class="truncate">${unnormalize(option)}</div>
                  </div>
                </button>
              </li>`;
            })}
          </ul>
          ${this.enteredValue.length > 0
            ? html`<div class="flex items-center justify-between py-1 px-4">
                <div class="truncate text-sm leading-[27px] ps-5">${this.enteredValue}</div>
                <button type="button" @click=${() => this._onClickOption()} class="btn-primary btn-mini">
                  Add
                </button>
              </div>`
            : ""}
        </div>
      </div>
      ${this.selected.map((option) => html`<input type="hidden" name="${this.name}[]" value="${option}" />`)}
    `;
  }
}
customElements.define("multi-select", MultiSelect);
