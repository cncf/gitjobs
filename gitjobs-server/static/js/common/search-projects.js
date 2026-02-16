import { html } from "/static/vendor/js/lit-all.v3.3.1.min.js";
import { unnormalize } from "/static/js/common/common.js";
import { triggerActionOnForm } from "/static/js/common/common.js";
import { LitWrapper } from "/static/js/common/lit-wrapper.js";
import { debounce } from "/static/js/common/common.js";

/**
 * Project search component with foundation filtering.
 * Allows searching and selecting projects from different foundations.
 * @extends LitWrapper
 */
export class SearchProjects extends LitWrapper {
  /**
   * Component properties definition
   * @property {Array} foundations - Available foundation options
   * @property {Array} selected - Currently selected projects
   * @property {string} enteredValue - Current search input value
   * @property {'cols'|'rows'} viewType - Display layout for selected items
   * @property {Array|null} visibleOptions - Filtered project suggestions
   * @property {boolean} visibleDropdown - Dropdown visibility state
   * @property {string} form - Form ID for input association
   * @property {'top'|'bottom'} alignment - Dropdown alignment
   * @property {number|null} activeIndex - Active suggestion index
   * @property {string|null} selectedFoundation - Selected foundation filter
   */
  static properties = {
    foundations: { type: Array },
    selected: { type: Array },
    enteredValue: { type: String },
    viewType: { type: String },
    visibleOptions: { type: Array | null },
    visibleDropdown: { type: Boolean },
    form: { type: String },
    alignment: { type: String },
    activeIndex: { type: Number | null },
    selectedFoundation: { type: String | null },
  };

  constructor() {
    super();
    this.foundations = [];
    this.selected = [];
    this.enteredValue = "";
    this.viewType = "cols";
    this.visibleOptions = null;
    this.visibleDropdown = false;
    this.form = "";
    this.alignment = "bottom";
    this.activeIndex = null;
    this.selectedFoundation = null;
    this._debouncedGetProjects = debounce(() => this._getProjects(), 300);
  }

  connectedCallback() {
    super.connectedCallback();
    window.addEventListener("mousedown", this._handleClickOutside);
  }

  disconnectedCallback() {
    super.disconnectedCallback();
    this._debouncedGetProjects.cancel?.();
    window.removeEventListener("mousedown", this._handleClickOutside);
  }

  /**
   * Public method to clear all selected projects and foundation.
   */
  async cleanSelected() {
    this.selected = [];
    this.selectedFoundation = null;

    // Wait for the update to complete
    await this.updateComplete;
  }

  /**
   * Fetches projects from server based on search and foundation.
   * @private
   */
  async _getProjects() {
    const requestValue = this.enteredValue;
    const requestFoundation = this.selectedFoundation;
    const url = `/projects/search?project=${encodeURIComponent(requestValue)}&foundation=${requestFoundation}`;
    try {
      const response = await fetch(url);
      if (!response.ok) {
        throw new Error(`Response status: ${response.status}`);
      }

      const json = await response.json();
      if (this.enteredValue !== requestValue || this.selectedFoundation !== requestFoundation) {
        return;
      }
      this.visibleOptions = json;
    } catch (error) {
      if (this.enteredValue !== requestValue || this.selectedFoundation !== requestFoundation) {
        return;
      }
      this.visibleOptions = [];
    } finally {
      if (this.enteredValue !== requestValue || this.selectedFoundation !== requestFoundation) {
        return;
      }
      this.visibleDropdown = true;
    }
  }

  /**
   * Handles foundation filter selection.
   * @param {Event} event - Change event
   * @private
   */
  _handleFoundationChange(event) {
    this._debouncedGetProjects.cancel?.();
    const selectedFoundation = event.target.value;
    if (selectedFoundation === "") {
      this.selectedFoundation = null;
    } else {
      this.selectedFoundation = selectedFoundation;
    }
    this.visibleOptions = null;
    this.enteredValue = "";
    this.visibleDropdown = false;
  }

  /**
   * Triggers project search when input is long enough.
   * @private
   */
  _filterOptions() {
    if (this.enteredValue.length > 2) {
      this._debouncedGetProjects();
    } else {
      this._debouncedGetProjects.cancel?.();
      this.visibleOptions = null;
      this.visibleDropdown = false;
      this.activeIndex = null;
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
   * Clears search input and hides dropdown.
   * @private
   */
  _cleanEnteredValue() {
    this._debouncedGetProjects.cancel?.();
    this.enteredValue = "";
    this.visibleDropdown = false;
    this.visibleOptions = null;
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
        if (this.activeIndex !== null && this.visibleOptions !== null && this.visibleOptions.length > 0) {
          const activeItem = this.visibleOptions[this.activeIndex];
          if (activeItem) {
            this._onSelect(activeItem);
          }
        }
        break;
      default:
        break;
    }
  }

  /**
   * Highlights suggestion item for keyboard navigation.
   * @param {'up'|'down'} direction - Navigation direction
   * @private
   */
  _highlightItem(direction) {
    if (this.visibleOptions && this.visibleOptions.length > 0) {
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
   * Selects a project and triggers form update.
   * @param {Object} value - Selected project object
   * @private
   */
  async _onSelect(value) {
    this._debouncedGetProjects.cancel?.();
    this.selected.push(value);
    this.enteredValue = "";
    this.visibleDropdown = false;
    this.visibleOptions = null;
    this.activeIndex = null;

    // Clear foundation filter when project is selected
    const foundationSelects = document.getElementsByName("foundation");
    foundationSelects.forEach((select) => {
      if (select.value !== "") {
        select.value = "";
      }
    });

    // Wait for the update to complete
    await this.updateComplete;

    // Trigger change event on the form
    triggerActionOnForm(this.form, "submit");
  }

  /**
   * Removes a selected project.
   * @param {string} name - Project name to remove
   * @private
   */
  async _onRemove(name) {
    this.selected = this.selected.filter((item) => item.name !== name);

    // Wait for the update to complete
    await this.updateComplete;

    // Trigger change event on the form
    triggerActionOnForm(this.form, "submit");
  }

  render() {
    const isDisabled = this.selectedFoundation === null;

    return html`<select
        class="select-primary py-0.5 text-[0.775rem]/6 text-stone-700 mb-2"
        @change=${this._handleFoundationChange}
      >
        <option value="" ?selected="${this.selectedFoundation === null}"></option>
        ${this.foundations.map((foundation) => {
          return html`<option
            value="${foundation.name}"
            ?selected="${this.selectedFoundation === foundation.name}"
          >
            ${foundation.name.toUpperCase()}
          </option>`;
        })}
      </select>
      <div class="mt-2 relative">
        <div class="absolute top-2 start-0 flex items-center ps-3 pointer-events-none">
          <div class="svg-icon size-3.5 icon-search bg-stone-300"></div>
        </div>
        <input
          @keydown="${this._handleKeyDown}"
          @input=${this._onInputChange}
          type="text"
          .value="${this.enteredValue}"
          class="input-primary py-0.5 peer ps-9 rounded-lg text-[0.775rem]/6 text-stone-700 ${isDisabled
            ? "opacity-50"
            : ""}"
          placeholder="Search projects"
          autocomplete="off"
          autocorrect="off"
          autocapitalize="off"
          spellcheck="false"
          autocomplete="off"
          ?disabled="${isDisabled}"
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
            ${this.visibleOptions !== null && this.visibleOptions.length > 0 && this.visibleDropdown
              ? html`<ul class="text-sm text-stone-700 overflow-auto max-h-[180px]">
                  ${this.visibleOptions.map((option, index) => {
                    const isSelected = this.selected.some(
                      (item) => item.name === option.name && item.foundation === option.foundation,
                    );
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
                        } capitalize block w-full text-left px-3 py-1`}
                        ?disabled="${isSelected}"
                      >
                        <div class="flex items-center space-x-3">
                          <div class="size-8 shrink-0 flex items-center justify-center">
                            <img
                              loading="lazy"
                              class="size-8 object-contain"
                              height="auto"
                              width="auto"
                              src="${option.logo_url}"
                              alt="${option.name} logo"
                            />
                          </div>
                          <div class="flex flex-col justify-start min-w-0">
                            <div class="truncate text-start text-xs/5 text-stone-700 font-medium">
                              ${option.name}
                            </div>
                            <div class="inline-flex">
                              <div
                                class="truncate text-nowrap uppercase max-w-[100%] text-[0.65rem] font-medium text-stone-500/75"
                              >
                                ${option.maturity}
                              </div>
                            </div>
                          </div>
                        </div>
                      </button>
                    </li>`;
                  })}
                </ul>`
              : html`<div class="px-8 py-4 text-sm/6 text-stone-600 italic">No projects found</div>`}
          </div>
        </div>
        ${this.selected.length > 0
          ? html`<div class="flex gap-2 mt-4 ${this.viewType === "rows" ? "flex-col" : "flex-wrap"}">
              ${this.selected.map(
                (opt, index) =>
                  html` <button
                      type="button"
                      @click=${() => this._onRemove(opt.name)}
                      class="inline-flex items-center justify-between ps-2 pe-1 py-1 bg-white border rounded-lg cursor-pointer select-none border-primary-500 text-primary-500 max-w-full group"
                    >
                      <div class="flex items-center justify-between space-x-3 w-full">
                        <div class="text-[0.8rem] text-center text-nowrap capitalize truncate">
                          <span class="uppercase text-[0.65rem] font-medium text-stone-500/75"
                            >${opt.foundation}:</span
                          >
                          ${unnormalize(opt.name)}
                        </div>
                        <div
                          class="svg-icon size-4 icon-close bg-stone-500 group-hover:bg-stone-800 shrink-0"
                        ></div>
                      </div>
                    </button>
                    <input
                      type="hidden"
                      form="${this.form}"
                      name="projects[${index}][name]"
                      value="${opt.name}"
                    />
                    <input
                      type="hidden"
                      form="${this.form}"
                      name="projects[${index}][foundation]"
                      value="${opt.foundation}"
                    />`,
              )}
            </div>`
          : ""}
      </div>`;
  }
}
customElements.define("search-projects", SearchProjects);
