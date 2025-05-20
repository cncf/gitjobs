import { html } from "/static/vendor/js/lit-all.v3.2.1.min.js";
import { unnormalize } from "/static/js/common/common.js";
import { triggerActionOnForm } from "/static/js/jobboard/filters.js";
import { LitWrapper } from "/static/js/common/lit-wrapper.js";
import { getBenefits } from "/static/js/common/data.js";

/**
 * SearchableFilter
 *
 * Custom element for a multi-select filter with search and dropdown.
 * Used for filtering lists with selectable options, supporting keyboard navigation.
 *
 * @property {string} name - The filter name, used for input and option source.
 * @property {Array} options - All available options for selection.
 * @property {Array} selectedOptions - Currently selected options.
 * @property {string} inputValue - Current value of the search input.
 * @property {string} layout - Layout style for selected options ("cols" or "rows").
 * @property {Array} filteredOptions - Options filtered by the search input.
 * @property {boolean} isDropdownOpen - Whether the dropdown is visible.
 * @property {string} formId - The form element's id for submitting changes.
 * @property {string} dropdownAlignment - Dropdown position ("top" or "bottom").
 * @property {number|null} highlightedIndex - Index of the highlighted option.
 */
export class SearchableFilter extends LitWrapper {
  static properties = {
    name: { type: String },
    options: { type: Array },
    selectedOptions: { type: Array },
    inputValue: { type: String },
    layout: { type: String },
    filteredOptions: { type: Array },
    isDropdownOpen: { type: Boolean },
    formId: { type: String },
    dropdownAlignment: { type: String },
    highlightedIndex: { type: Number | null },
  };

  /**
   * Initializes default property values.
   */
  constructor() {
    super();
    this.name = "name";
    this.options = [];
    this.selectedOptions = [];
    this.inputValue = "";
    this.layout = "cols";
    this.filteredOptions = [];
    this.isDropdownOpen = false;
    this.formId = "";
    this.dropdownAlignment = "bottom";
    this.highlightedIndex = null;
  }

  /**
   * Lifecycle: Called when element is added to the DOM.
   * Sets up outside click handler and initializes options.
   */
  connectedCallback() {
    super.connectedCallback();
    window.addEventListener("mousedown", this._handleOutsideClick);
    this._initializeOptions();
  }

  /**
   * Lifecycle: Called when element is removed from the DOM.
   * Removes outside click handler.
   */
  disconnectedCallback() {
    super.disconnectedCallback();
    window.removeEventListener("mousedown", this._handleOutsideClick);
  }

  /**
   * Clears all selected options and waits for update.
   * @returns {Promise<void>}
   */
  async clearSelectedOptions() {
    this.selectedOptions = [];
    await this.updateComplete;
  }

  /**
   * Initializes options based on filter name.
   * For "benefits", loads options from getBenefits().
   */
  _initializeOptions() {
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
   * Filters options based on inputValue.
   * Updates filteredOptions with matching results.
   */
  _filterOptions() {
    if (this.inputValue.length > 0) {
      this.filteredOptions = this.options.filter((option) => {
        const name = unnormalize(option);
        return name.toLowerCase().includes(this.inputValue.toLowerCase());
      });
    } else {
      this.filteredOptions = this.options;
    }
  }

  /**
   * Handles input change event for the search box.
   * Updates inputValue and filteredOptions.
   * @param {Event} event - Input event
   */
  _handleInputChange(event) {
    this.inputValue = event.target.value;
    this._filterOptions();
  }

  /**
   * Clears the search input, closes dropdown, resets highlight and filters.
   */
  _cleanInputValue() {
    this.inputValue = "";
    this.isDropdownOpen = false;
    this._filterOptions();
    this.highlightedIndex = null;
  }

  /**
   * Handles clicks outside the component to close dropdown and clear input.
   * @param {MouseEvent} e - Mouse event
   */
  _handleOutsideClick = (e) => {
    if (!this.contains(e.target)) {
      this._cleanInputValue();
    }
  };

  /**
   * Handles keyboard navigation and selection in the dropdown.
   * Supports ArrowDown, ArrowUp, and Enter keys.
   * @param {KeyboardEvent} event - Keydown event
   */
  _handleKeyDown(event) {
    switch (event.key) {
      case "ArrowDown":
        this._highlightOption("down");
        break;
      case "ArrowUp":
        this._highlightOption("up");
        break;
      case "Enter":
        event.preventDefault();
        if (this.highlightedIndex !== null && this.filteredOptions.length > 0) {
          const activeItem = this.filteredOptions[this.highlightedIndex];
          if (activeItem) {
            this._selectOption(activeItem);
          }
        }
        break;
      default:
        break;
    }
  }

  /**
   * Moves the highlight up or down in the filtered options list.
   * @param {"up"|"down"} direction - Direction to move highlight
   */
  _highlightOption(direction) {
    if (this.options && this.options.length > 0) {
      if (this.highlightedIndex === null) {
        this.highlightedIndex = direction === "down" ? 0 : this.options.length - 1;
      } else {
        let newIndex = direction === "down" ? this.highlightedIndex + 1 : this.highlightedIndex - 1;
        if (newIndex >= this.options.length) {
          newIndex = 0;
        }
        if (newIndex < 0) {
          newIndex = this.options.length - 1;
        }
        this.highlightedIndex = newIndex;
      }
    }
  }

  /**
   * Selects an option, adds it to selectedOptions, clears input, closes dropdown,
   * and triggers form submit.
   * @param {*} value - Option value to select
   * @returns {Promise<void>}
   */
  async _selectOption(value) {
    this.selectedOptions.push(value);
    this.inputValue = "";
    this.isDropdownOpen = false;
    this._filterOptions();
    this.highlightedIndex = null;
    await this.updateComplete;
    triggerActionOnForm(this.formId, "submit");
  }

  /**
   * Removes an option from selectedOptions and triggers form submit.
   * @param {*} value - Option value to remove
   * @returns {Promise<void>}
   */
  async _removeOption(value) {
    this.selectedOptions = this.selectedOptions.filter((item) => item !== value);
    await this.updateComplete;
    triggerActionOnForm(this.formId, "submit");
  }

  /**
   * Renders the searchable filter input, dropdown, and selected options.
   * @returns {import("lit").TemplateResult}
   */
  render() {
    return html`<div class="mt-2 relative">
      <div class="absolute top-2 start-0 flex items-center ps-3 pointer-events-none">
        <div class="svg-icon size-3.5 icon-search bg-stone-300"></div>
      </div>
      <input
        type="text"
        @keydown="${this._handleKeyDown}"
        @input=${this._handleInputChange}
        @focus=${() => (this.isDropdownOpen = true)}
        .value="${this.inputValue}"
        class="input-primary py-0.5 peer ps-9 rounded-lg text-[0.8rem]/6"
        placeholder="Search ${this.name}"
        autocomplete="off"
        autocorrect="off"
        autocapitalize="off"
        spellcheck="false"
      />
      <div class="absolute end-1.5 top-0.5 peer-placeholder-shown:hidden">
        <button @click=${this._cleanInputValue} type="button" class="cursor-pointer mt-[2px]">
          <div class="svg-icon size-5 bg-stone-400 hover:bg-stone-700 icon-close"></div>
        </button>
      </div>
      <div
        class="absolute z-10 start-0 end-0 ${this.dropdownAlignment === "top"
          ? "-top-[193px] h-[186px]"
          : ""}"
      >
        <div
          class="${this.dropdownAlignment === "top" ? "h-full" : ""} ${!this.isDropdownOpen
            ? "hidden"
            : ""} bg-white divide-y divide-stone-100 rounded-lg shadow w-full border border-stone-200 mt-1"
        >
          ${this.filteredOptions.length > 0 && this.isDropdownOpen
            ? html`<ul class="text-sm text-stone-700 overflow-auto max-h-[180px]">
                ${this.filteredOptions.map((option, index) => {
                  const isSelected = this.selectedOptions.includes(option);
                  return html`<li
                    class="group ${this.highlightedIndex === index ? "active" : ""}"
                    data-index="${index}"
                  >
                    <button
                      type="button"
                      @click=${() => this._selectOption(option)}
                      @mouseover=${() => (this.highlightedIndex = index)}
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
      ${this.selectedOptions.length > 0
        ? html`<div class="flex gap-2 mt-4 ${this.layout === "rows" ? "flex-col" : "flex-wrap"}">
            ${this.selectedOptions.map(
              (opt) =>
                html` <button
                    type="button"
                    @click=${() => this._removeOption(opt)}
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
                  <input type="hidden" form="${this.formId}" name="${this.name}[]" value="${opt}" />`,
            )}
          </div>`
        : ""}
    </div>`;
  }
}

/**
 * Registers the SearchableFilter component as a custom element.
 */
customElements.define("searchable-filter", SearchableFilter);
