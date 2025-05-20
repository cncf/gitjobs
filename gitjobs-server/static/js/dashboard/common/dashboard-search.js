import { html } from "/static/vendor/js/lit-all.v3.2.1.min.js";
import { LitWrapper } from "/static/js/common/lit-wrapper.js";
import { debounce } from "/static/js/common/common.js";

/**
 * DashboardSearch web component for searching and selecting projects or foundation members.
 * Extends LitWrapper for reactive rendering.
 *
 * @class DashboardSearch
 * @property {string} type - Type of search ("projects" or "members").
 * @property {Array} foundations - List of available foundations for filtering.
 * @property {Array} selectedOptions - Currently selected projects or members.
 * @property {string} inputValue - Current value of the search input.
 * @property {string} layout - Layout style for the component (e.g., "cols").
 * @property {Array} visibleOptions - Options currently visible in the dropdown.
 * @property {boolean} isDropdownOpen - Whether the dropdown is open.
 * @property {number|null} highlightedIndex - Index of the highlighted dropdown option.
 * @property {string} selectedFoundation - Currently selected foundation filter.
 */
export class DashboardSearch extends LitWrapper {
  static properties = {
    type: { type: String },
    foundations: { type: Array },
    selectedOptions: { type: Array },
    inputValue: { type: String },
    layout: { type: String },
    visibleOptions: { type: Array },
    isDropdownOpen: { type: Boolean },
    highlightedIndex: { type: Number | null },
    selectedFoundation: { type: String },
  };

  // Default foundation filter value
  defaultFoundation = "cncf";

  /**
   * Initializes component state and default property values.
   */
  constructor() {
    super();
    this.type = "projects";
    this.foundations = [];
    this.selectedOptions = [];
    this.inputValue = "";
    this.layout = "cols";
    this.visibleOptions = [];
    this.isDropdownOpen = false;
    this.highlightedIndex = null;
    this.selectedFoundation = this.defaultFoundation;
  }

  /**
   * Lifecycle: Called when component is added to the DOM.
   * Adds event listener for outside clicks to close dropdown.
   */
  connectedCallback() {
    super.connectedCallback();
    window.addEventListener("mousedown", this._handleOutsideClick);
  }

  /**
   * Lifecycle: Called when component is removed from the DOM.
   * Removes event listener for outside clicks.
   */
  disconnectedCallback() {
    super.disconnectedCallback();
    window.removeEventListener("mousedown", this._handleOutsideClick);
  }

  /**
   * Fetches search options from the server based on input and foundation filter if applicable.
   * Updates visibleOptions with the fetched results.
   */
  async _fetchOptions() {
    const url = `${this.type === "members" ? "/dashboard/members/search?member=" : "/projects/search?project="}${encodeURIComponent(this.inputValue)}&foundation=${this.selectedFoundation}`;
    try {
      const response = await fetch(url);
      if (!response.ok) {
        throw new Error(`Response status: ${response.status}`);
      }
      const json = await response.json();
      this.visibleOptions = json;
    } catch (error) {
      // TODO - Handle error (e.g., show notification)
    } finally {
      this.isDropdownOpen = true;
    }
  }

  /**
   * Handles changes to the foundation filter dropdown.
   * Resets search input and visible options.
   * @param {Event} event - Change event from foundation select.
   */
  _handleFoundationChange(event) {
    const selectedFoundation = event.target.value;
    if (selectedFoundation === "") {
      this.selectedFoundation = this.defaultFoundation;
    } else {
      this.selectedFoundation = selectedFoundation;
    }
    this.visibleOptions = [];
    this.inputValue = "";
    this.isDropdownOpen = false;
    this.highlightedIndex = null;
  }

  /**
   * Filters options based on input value.
   * Triggers fetch if input length > 2, otherwise clears options.
   * Uses debounce to limit fetch frequency.
   */
  _filterOptions() {
    if (this.inputValue.length > 2) {
      debounce(this._fetchOptions(), 300);
    } else {
      this.visibleOptions = [];
      this.isDropdownOpen = false;
      this.highlightedIndex = null;
    }
  }

  /**
   * Handles input changes in the search box.
   * Updates inputValue and triggers filtering.
   * @param {Event} event - Input event from search box.
   */
  _handleInputChange(event) {
    this.inputValue = event.target.value;
    this._filterOptions();
  }

  /**
   * Clears the search input and resets dropdown/options.
   */
  _cleanInputValue() {
    this.inputValue = "";
    this.isDropdownOpen = false;
    this.visibleOptions = [];
    this.highlightedIndex = null;
    this.selectedFoundation = this.defaultFoundation;
  }

  /**
   * Handles clicks outside the component to close dropdown and reset input.
   * @param {MouseEvent} e - Mouse event.
   */
  _handleOutsideClick = (e) => {
    if (!this.contains(e.target)) {
      this._cleanInputValue();
    }
  };

  /**
   * Handles keyboard navigation in the dropdown.
   * Supports ArrowDown, ArrowUp, and Enter for selection.
   * @param {KeyboardEvent} event - Keyboard event.
   */
  _handleKeyDown(event) {
    switch (event.key) {
      // Highlight the next item in the list
      case "ArrowDown":
        this._highlightOption("down");
        break;
      // Highlight the previous item in the list
      case "ArrowUp":
        this._highlightOption("up");
        break;
      // Select the highlighted item
      case "Enter":
        event.preventDefault();
        if (this.highlightedIndex !== null && this.visibleOptions.length > 0) {
          const activeItem = this.visibleOptions[this.highlightedIndex];
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
   * Highlights the next or previous option in the dropdown.
   * @param {String} direction - "down" or "up"
   */
  _highlightOption(direction) {
    if (this.visibleOptions.length > 0) {
      if (this.highlightedIndex === null) {
        this.highlightedIndex = direction === "down" ? 0 : this.visibleOptions.length - 1;
      } else {
        let newIndex = direction === "down" ? this.highlightedIndex + 1 : this.highlightedIndex - 1;
        if (newIndex >= this.visibleOptions.length) {
          newIndex = 0;
        }
        if (newIndex < 0) {
          newIndex = this.visibleOptions.length - 1;
        }
        this.highlightedIndex = newIndex;
      }
    }
  }

  /**
   * Selects an option from the dropdown.
   * Adds to selectedOptions and resets input/dropdown.
   * @param {Object} item - Selected project or member.
   */
  _selectOption(item) {
    if (this.type === "projects") {
      this.selectedOptions.push(item);
    } else {
      this.selectedOptions = [item];
    }
    this.inputValue = "";
    this.isDropdownOpen = false;
    this.visibleOptions = [];
    this.highlightedIndex = null;
  }

  /**
   * Removes a selected option by its ID.
   * @param {String|Number} id - ID of the option to remove.
   */
  _removeOption(id) {
    this.selectedOptions = this.selectedOptions.filter((item) => {
      const itemId = this.type === "members" ? item.member_id : item.project_id;
      return itemId !== id;
    });
  }

  /**
   * Renders the dashboard-search component UI.
   * Includes foundation filter, search input, dropdown, and selected options.
   * @returns {TemplateResult}
   */
  render() {
    return html`<div>
        <label for="project" class="form-label"
          >${this.type === "members" ? "Foundation member" : "Projects"}</label
        >
        <div class="grid grid-cols-1 gap-x-6 gap-y-8 md:grid-cols-6 max-w-5xl">
          <div class="mt-2 col-span-full lg:col-span-2">
            <select class="select-primary uppercase" @change=${this._handleFoundationChange}>
              ${this.foundations.map((foundation) => {
                return html`<option
                  value="${foundation.name}"
                  ?selected="${this.selectedFoundation === foundation.name}"
                >
                  ${foundation.name.toUpperCase()}
                </option>`;
              })}
            </select>
          </div>

          <div class="col-span-full lg:col-span-4">
            <div class="mt-2 relative">
              <div class="absolute top-2.5 start-0 flex items-center ps-3 pointer-events-none">
                <div class="svg-icon size-4 icon-search bg-stone-300"></div>
              </div>
              <input
                type="text"
                @keydown="${this._handleKeyDown}"
                @input=${this._handleInputChange}
                .value="${this.inputValue}"
                class="input-primary peer ps-10"
                placeholder="Search ${this.type}"
                autocomplete="off"
                autocorrect="off"
                autocapitalize="off"
                spellcheck="false"
              />
              <div class="absolute end-1.5 top-1.5 peer-placeholder-shown:hidden">
                <button @click=${this._cleanInputValue} type="button" class="cursor-pointer mt-[2px]">
                  <div class="svg-icon size-5 bg-stone-400 hover:bg-stone-700 icon-close"></div>
                </button>
              </div>
              <div class="absolute z-10 start-0 end-0">
                <div
                  class="${!this.isDropdownOpen
                    ? "hidden"
                    : ""} bg-white rounded-lg shadow w-full border border-stone-200 mt-1"
                >
                  ${this.visibleOptions.length > 0 && this.isDropdownOpen
                    ? html`<ul class="text-sm text-stone-700 overflow-auto max-h-[180px]">
                        ${this.visibleOptions.map((option, index) => {
                          const isSelected = this.selectedOptions.some(
                            (item) => item.name === option.name && item.foundation === option.foundation,
                          );
                          return html`<li
                            class="group ${index > 0 ? "border-t border-stone-200" : ""} ${this
                              .highlightedIndex === index
                              ? "active"
                              : ""}"
                          >
                            <button
                              type="button"
                              @click=${() => this._selectOption(option)}
                              @mouseover=${() => (this.highlightedIndex = index)}
                              class=${`px-4 py-2 w-full ${
                                isSelected
                                  ? "bg-stone-100 opacity-50"
                                  : "cursor-pointer hover:bg-stone-100 group-[.active]:bg-stone-100"
                              }`}
                              ?disabled="${isSelected}"
                            >
                              <div class="flex items-center space-x-3">
                                <div class="flex justify-center items-center shrink-0 size-8 lg:size-10">
                                  <img
                                    loading="lazy"
                                    class="size-8 lg:size-10 object-contain"
                                    height="auto"
                                    width="auto"
                                    src="${option.logo_url}"
                                    alt="${option.name} logo"
                                  />
                                </div>
                                <div class="flex flex-col justify-start min-w-0">
                                  <div class="truncate text-start text-stone-700 font-medium">
                                    ${option.name}
                                  </div>
                                  <div class="inline-flex">
                                    <div
                                      class="truncate text-nowrap uppercase max-w-[100%] text-xs/6 font-medium text-stone-500/75"
                                    >
                                      ${option.foundation}
                                      ${this.type === "projects" ? option.maturity : `${option.level} member`}
                                    </div>
                                  </div>
                                </div>
                              </div>
                            </button>
                          </li>`;
                        })}
                      </ul>`
                    : html`<div class="px-8 py-4 text-sm/6 text-stone-600 italic">
                        No ${this.type} found
                      </div>`}
                </div>
              </div>
            </div>
          </div>
        </div>
        <p class="form-legend">
          ${this.type === "projects"
            ? "If the job position involves contributing to any of the supported foundations projects, please list them here."
            : "If your company is a member of any of the supported foundations please select the corresponding member entry. Jobs posted by members will be featured across the site. False membership claims may lead to the suspension of the employer and associated user accounts."}
        </p>
      </div>
      <div class="col-span-full mt-4">
        ${this.selectedOptions.length > 0
          ? html` <div class="flex flex-wrap gap-5 w-full">
              ${this.selectedOptions.map(
                (opt, index) =>
                  html`<div class="relative border border-stone-200 rounded-lg p-4 pe-10 bg-white min-w-64">
                    <button
                      @click=${() =>
                        this._removeOption(this.type === "members" ? opt.member_id : opt.project_id)}
                      type="button"
                      class="rounded-full cursor-pointer bg-stone-100 hover:bg-stone-200 absolute top-1 end-1"
                    >
                      <div class="svg-icon size-5 bg-stone-400 hover:bg-stone-700 icon-close"></div>
                    </button>
                    <div class="flex items-center space-x-3">
                      <div class="size-10 shrink-0 flex items-center justify-center">
                        <img
                          class="size-10 object-contain"
                          height="auto"
                          width="auto"
                          src="${opt.logo_url}"
                          alt="${opt.name} logo"
                        />
                      </div>
                      <div class="flex flex-col justify-start min-w-0">
                        <div class="truncate text-start text-stone-700 font-medium ">${opt.name}</div>
                        <div class="inline-flex">
                          <div
                            class="truncate text-nowrap uppercase max-w-[100%] text-xs/6 font-medium text-stone-500/75"
                          >
                            ${opt.foundation}
                            ${this.type === "members" ? `${opt.level} member` : opt.maturity}
                          </div>
                        </div>
                      </div>
                    </div>
                    ${this.type === "projects"
                      ? html`<input
                            type="hidden"
                            name="projects[${index}][project_id]"
                            value="${opt.project_id}"
                          />
                          <input type="hidden" name="projects[${index}][name]" value="${opt.name}" />
                          <input type="hidden" name="projects[${index}][maturity]" value="${opt.maturity}" />
                          <input
                            type="hidden"
                            name="projects[${index}][foundation]"
                            value="${opt.foundation}"
                          />
                          <input type="hidden" name="projects[${index}][logo_url]" value="${opt.logo_url}" />`
                      : html`<input type="hidden" name="member[member_id]" value="${opt.member_id}" />
                          <input type="hidden" name="member[name]" value="${opt.name}" />
                          <input type="hidden" name="member[level]" value="${opt.level}" />
                          <input type="hidden" name="member[foundation]" value="${opt.foundation}" />
                          <input type="hidden" name="member[logo_url]" value="${opt.logo_url}" />`}
                  </div> `,
              )}
            </div>`
          : ""}
      </div>`;
  }
}

/**
 * Registers the DashboardSearch component as a custom element.
 */
customElements.define("dashboard-search", DashboardSearch);
