import { html } from "/static/vendor/js/lit-all.v3.3.1.min.js";
import { unnormalize } from "/static/js/common/common.js";
import { triggerActionOnForm } from "/static/js/jobboard/filters.js";
import { LitWrapper } from "/static/js/common/lit-wrapper.js";
import { getBenefits } from "/static/js/common/data.js";

/**
 * Searchable filter component for selectable options.
 * Supports search, multi-select, and keyboard navigation.
 * @extends LitWrapper
 */
export class SearchableFilter extends LitWrapper {
  /**
   * Component properties definition
   * @property {string} name - Filter name (e.g., 'benefits')
   * @property {string[]} options - Available filter options
   * @property {string[]} selected - Currently selected options
   * @property {string} enteredValue - Current search input value
   * @property {'cols'|'rows'} viewType - Display layout for selected items
   * @property {string[]} visibleOptions - Filtered options based on search
   * @property {boolean} visibleDropdown - Dropdown visibility state
   * @property {string} form - Form ID for input association
   * @property {'top'|'bottom'} alignment - Dropdown alignment
   * @property {number|null} activeIndex - Active suggestion index
   */
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
    activeIndex: { type: Number | null },
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
    this.activeIndex = null;
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
   * Public method to clear all selected options.
   */
  async cleanSelected() {
    this.selected = [];

    // Wait for the update to complete
    await this.updateComplete;
  }

  /**
   * Loads options based on filter name.
   * @private
   */
  _getOptions() {
    switch (this.name) {
      case "benefits":
        this.options = getBenefits();
        break;
      default:
        this.options = this.options;
    }

    this._filterOptions();
  }

  /**
   * Filters visible options based on search input.
   * @private
   */
  _filterOptions() {
    if (this.enteredValue.length > 0) {
      this.visibleOptions = this.options.filter((option) => {
        const name = unnormalize(option);
        return name.toLowerCase().includes(this.enteredValue.toLowerCase());
      });
    } else {
      this.visibleOptions = this.options;
    }
  }

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
   * Clears search input and resets filter state.
   * @private
   */
  _cleanEnteredValue() {
    this.enteredValue = "";
    this.visibleDropdown = false;
    this._filterOptions();
    this.activeIndex = null;
  }

  /**
   * Handles click outside to close dropdown.
   * @param {MouseEvent} event - The click event
   * @private
   */
  _handleClickOutside = (event) => {
    if (!this.contains(event.target)) {
      this._cleanEnteredValue();
    }
  };

  /**
   * Handles keyboard navigation and selection.
   * @param {KeyboardEvent} event - Keyboard event
   * @private
   */
  _handleKeyDown(event) {
    switch (event.key) {
      // Highlight the next item in the list
      case "ArrowDown":
        this._highlightItem("down");
        break;
      // Highlight the previous item in the list
      case "ArrowUp":
        this._highlightItem("up");
        break;
      // Select the highlighted item
      case "Enter":
        event.preventDefault();
        if (this.activeIndex !== null && this.visibleOptions.length > 0) {
          const activeItem = this.visibleOptions[this.activeIndex];
          if (activeItem) {
            const activeItem = this.visibleOptions[this.activeIndex];
            this._onSelect(activeItem);
          }
        }
        break;
      default:
        break;
    }
  }

  /**
   * Highlights option for keyboard navigation.
   * @param {'up'|'down'} direction - Navigation direction
   * @private
   */
  _highlightItem(direction) {
    if (this.options && this.options.length > 0) {
      if (this.activeIndex === null) {
        this.activeIndex = direction === "down" ? 0 : this.visibleOptions.length - 1;
      } else {
        let newIndex = direction === "down" ? this.activeIndex + 1 : this.activeIndex - 1;
        if (newIndex >= this.visibleOptions.length) {
          newIndex = 0;
        }
        if (newIndex < 0) {
          newIndex = this.visibleOptions.length - 1;
        }
        this.activeIndex = newIndex;
      }
    }
  }

  /**
   * Adds option to selected list and triggers form update.
   * @param {string} value - Option to select
   * @private
   */
  async _onSelect(value) {
    this.selected.push(value);
    this.enteredValue = "";
    this.visibleDropdown = false;
    this._filterOptions();
    this.activeIndex = null;

    // Wait for the update to complete
    await this.updateComplete;

    // Trigger change event on the form
    triggerActionOnForm(this.form, "submit");
  }

  /**
   * Removes option from selected list.
   * @param {string} value - Option to remove
   * @private
   */
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
        @keydown="${this._handleKeyDown}"
        @input=${this._onInputChange}
        @focus=${() => (this.visibleDropdown = true)}
        .value="${this.enteredValue}"
        class="input-primary py-0.5 peer ps-9 rounded-lg text-[0.775rem]/6 text-stone-700"
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
                ${this.visibleOptions.map((option, index) => {
                  const isSelected = this.selected.includes(option);
                  return html`<li
                    class="group ${this.activeIndex === index ? "active" : ""}"
                    data-index="${index}"
                  >
                    <button
                      type="button"
                      @click=${() => this._onSelect(option)}
                      @mouseover=${() => (this.activeIndex = index)}
                      class=${`group-[.active]:bg-stone-100 ${
                        isSelected ? "bg-stone-100 opacity-50" : "cursor-pointer hover:bg-stone-100"
                      } capitalize block w-full text-left px-4 py-1`}
                      ?disabled="${isSelected}"
                    >
                      <div class="flex items-center">
                        <div class="size-3 me-2">
                          ${isSelected
                            ? html`<div class="svg-icon size-3 icon-check bg-stone-400"></div>`
                            : ""}
                        </div>
                        <div class="truncate text-[0.8rem]/6">${unnormalize(option)}</div>
                      </div>
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
                  <input type="hidden" form="${this.form}" name="${this.name}" value="${opt}" />`,
            )}
          </div>`
        : ""}
    </div>`;
  }
}
customElements.define("searchable-filter", SearchableFilter);
