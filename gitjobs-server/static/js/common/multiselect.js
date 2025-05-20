/**
 * MultiSelect web component for selecting multiple items from a dropdown list.
 * Extends LitWrapper for reactive rendering.
 */

import { html } from "/static/vendor/js/lit-all.v3.2.1.min.js";
import { unnormalize } from "/static/js/common/common.js";
import { LitWrapper } from "/static/js/common/lit-wrapper.js";
import { getBenefits, getSkills } from "/static/js/common/data.js";

/**
 * @class MultiSelect
 * @property {string} name - Name attribute for the input.
 * @property {string} label - Label text for the input.
 * @property {Array} items - List of all selectable items.
 * @property {Array} selectedItems - List of currently selected items.
 * @property {string} inputValue - Current value in the search input.
 * @property {Array} filteredItems - Items filtered by the search input.
 * @property {boolean} isDropdownOpen - Dropdown visibility state.
 * @property {string} legend - Optional legend/help text.
 */
export class MultiSelect extends LitWrapper {
  static properties = {
    name: { type: String },
    label: { type: String },
    items: { type: Array },
    selectedItems: { type: Array },
    inputValue: { type: String },
    filteredItems: { type: Array },
    isDropdownOpen: { type: Boolean },
    legend: { type: String },
  };

  /**
   * Initializes the MultiSelect component with default values.
   */
  constructor() {
    super();
    this.name = "name";
    this.label = "label";
    this.items = [];
    this.selectedItems = [];
    this.inputValue = "";
    this.filteredItems = [];
    this.isDropdownOpen = false;
    this.legend = undefined;
  }

  /**
   * Lifecycle: Called when the component is added to the DOM.
   * Adds outside click listener and initializes items.
   */
  connectedCallback() {
    super.connectedCallback();
    window.addEventListener("mousedown", this.handleOutsideClick);
    this._initializeItems();
  }

  /**
   * Lifecycle: Called when the component is removed from the DOM.
   * Removes outside click listener.
   */
  disconnectedCallback() {
    super.disconnectedCallback();
    window.removeEventListener("mousedown", this.handleOutsideClick);
  }

  /**
   * Filters items based on the current input value.
   * Updates filteredItems with matching results.
   */
  _filterVisibleItems() {
    if (this.inputValue.length > 0) {
      this.filteredItems = this.items.filter((item) =>
        unnormalize(item).toLowerCase().includes(this.inputValue.toLowerCase()),
      );
    } else {
      this.filteredItems = this.items;
    }
  }

  /**
   * Initializes the items list based on the name property.
   * Populates items and filteredItems.
   */
  _initializeItems() {
    switch (this.name) {
      case "benefits":
        this.items = getBenefits();
        break;
      case "skills":
        this.items = getSkills();
        break;
      default:
        this.items = this.items;
    }
    this._filterVisibleItems();
  }

  /**
   * Handles input changes in the search box.
   * Updates inputValue and filters items.
   * @param {Event} event - Input event from the search box.
   */
  _handleInputChange(event) {
    const { value } = event.target;
    this.inputValue = value;
    this._filterVisibleItems();
  }

  /**
   * Removes an item from the selectedItems list.
   * @param {string} item - Item to remove.
   */
  _removeSelectedItem(item) {
    this.selectedItems = this.selectedItems.filter((selectedItem) => selectedItem !== item);
  }

  /**
   * Adds an item to the selectedItems list.
   * If no item is provided, adds the current inputValue.
   * Clears input and closes dropdown.
   * @param {string} [item] - Item to add.
   */
  _addSelectedItem(item) {
    this.selectedItems.push(item || this.inputValue);
    this.inputValue = "";
    this.isDropdownOpen = false;
    this._filterVisibleItems();
  }

  /**
   * Handles clicks outside the component to close the dropdown.
   * @param {MouseEvent} e - Mouse event.
   */
  handleOutsideClick = (e) => {
    if (!this.contains(e.target)) {
      this.isDropdownOpen = false;
    }
  };

  /**
   * Renders the MultiSelect component template.
   * @returns {import('lit').TemplateResult}
   */
  render() {
    return html`
      <label for="${this.name}" class="form-label">${this.label}</label>
      <div class="mt-2 relative">
        <div
          class="input-primary px-1.5 flex flex-wrap focus-within:outline-[3px] focus-within:-outline-offset-2 focus-within:outline-primary-600"
        >
          <div class="flex flex-items flex-wrap w-full gap-2">
            ${this.selectedItems.map(
              (item) =>
                html`<span
                  class="inline-flex items-center text-nowrap max-w-[100%] ps-2 pe-0.5 py-0.5 text-xs font-medium text-stone-800 bg-stone-100 rounded-full"
                >
                  <div class="flex items-center w-full">
                    <div class="truncate uppercase">${unnormalize(item)}</div>
                    <button
                      type="button"
                      @click=${() => this._removeSelectedItem(item)}
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
              @input=${this._handleInputChange}
              @focus=${() => (this.isDropdownOpen = true)}
              .value="${this.inputValue}"
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
            !this.isDropdownOpen ? "hidden" : ""
          } absolute start-0 z-10 bg-white divide-y divide-stone-100 rounded-lg shadow w-full border border-stone-200 mt-1 ${
            this.legend ? "top-10" : ""
          }`}
        >
          <ul class="text-sm text-stone-700 overflow-x-auto max-h-[150px]">
            ${this.filteredItems.map((item) => {
              const isSelected = this.selectedItems.includes(item);
              return html`<li class="group">
                <button
                  @click=${() => this._addSelectedItem(item)}
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
                    <div class="truncate">${unnormalize(item)}</div>
                  </div>
                </button>
              </li>`;
            })}
          </ul>
          ${this.inputValue.length > 0
            ? html`<div class="flex items-center justify-between py-1 px-4">
                <div class="truncate text-sm leading-[27px] ps-5">${this.inputValue}</div>
                <button type="button" @click=${() => this._addSelectedItem()} class="btn-primary btn-mini">
                  Add
                </button>
              </div>`
            : ""}
        </div>
      </div>
      ${this.selectedItems.map((item) => html`<input type="hidden" name="${this.name}[]" value="${item}" />`)}
    `;
  }
}

/**
 * Registers the MultiSelect component as a custom element.
 */
customElements.define("multi-select", MultiSelect);
