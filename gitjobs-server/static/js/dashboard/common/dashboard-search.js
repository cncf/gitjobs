import { html } from "/static/vendor/js/lit-all.v3.2.1.min.js";
import { LitWrapper } from "/static/js/common/lit-wrapper.js";
import { debounce } from "/static/js/common/common.js";

/**
 * Dashboard search component for projects and foundation members.
 * Supports dropdown selection, autocomplete, and multi-select.
 * @extends LitWrapper
 */
export class DashboardSearch extends LitWrapper {
  /**
   * Component properties definition
   * @property {'projects'|'members'} type - Search type
   * @property {Array} foundations - Available foundation options
   * @property {Array} selected - Currently selected items
   * @property {string} enteredValue - Current search input value
   * @property {Array} visibleOptions - Filtered suggestions
   * @property {boolean} visibleDropdown - Dropdown visibility state
   * @property {number|null} activeIndex - Active suggestion index
   * @property {string} selectedFoundation - Selected foundation filter
   */
  static properties = {
    type: { type: String },
    foundations: { type: Array },
    selected: { type: Array },
    enteredValue: { type: String },
    visibleOptions: { type: Array },
    visibleDropdown: { type: Boolean },
    activeIndex: { type: Number | null },
    selectedFoundation: { type: String },
  };

  /** @type {string} Default foundation when none selected */
  defaultFoundation = "cncf";

  constructor() {
    super();
    this.type = "projects";
    this.foundations = [];
    this.selected = [];
    this.enteredValue = "";
    this.viewType = "cols";
    this.visibleOptions = [];
    this.visibleDropdown = false;
    this.activeIndex = null;
    this.selectedFoundation = this.defaultFoundation;
  }

  connectedCallback() {
    super.connectedCallback();
    window.addEventListener("mousedown", this._handleClickOutside);
  }

  disconnectedCallback() {
    super.disconnectedCallback();
    window.addEventListener("mousedown", this._handleClickOutside);
  }

  /**
   * Fetches projects or members from server based on search criteria.
   * @private
   */
  async _getProjects() {
    const url = `${this.type === "members" ? "/dashboard/members/search?member=" : "/projects/search?project="}${encodeURIComponent(this.enteredValue)}&foundation=${this.selectedFoundation}`;
    try {
      const response = await fetch(url);
      if (!response.ok) {
        throw new Error(`Response status: ${response.status}`);
      }

      const json = await response.json();
      this.visibleOptions = json;
    } catch (error) {
      // TODO: Implement error handling
    } finally {
      this.visibleDropdown = true;
    }
  }

  /**
   * Handles foundation filter selection changes.
   * @param {Event} event - Change event
   * @private
   */
  _handleFoundationChange(event) {
    const selectedFoundation = event.target.value;
    if (selectedFoundation === "") {
      this.selectedFoundation = this.defaultFoundation;
    } else {
      this.selectedFoundation = selectedFoundation;
    }
    this.visibleOptions = [];
    this.enteredValue = "";
    this.visibleDropdown = false;
    this.activeIndex = null;
  }

  /**
   * Triggers search when input is long enough.
   * @private
   */
  _filterOptions() {
    if (this.enteredValue.length > 2) {
      debounce(this._getProjects(this.enteredValue), 300);
    } else {
      this.visibleOptions = [];
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
   * Resets search input and related state.
   * @private
   */
  _cleanEnteredValue() {
    this.enteredValue = "";
    this.visibleDropdown = false;
    this.visibleOptions = [];
    this.activeIndex = null;
    this.selectedFoundation = this.defaultFoundation;
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
    if (this.visibleOptions.length > 0) {
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
   * Selects an item from suggestions.
   * @param {Object} item - Selected item object
   * @private
   */
  _onSelect(item) {
    if (this.type === "projects") {
      this.selected.push(item);
    } else {
      this.selected = [item];
    }
    this.enteredValue = "";
    this.visibleDropdown = false;
    this.visibleOptions = [];
    this.activeIndex = null;
  }

  /**
   * Removes a selected item.
   * @param {string} id - Item ID to remove
   * @private
   */
  _onRemove(id) {
    this.selected = this.selected.filter((item) => {
      const itemId = this.type === "members" ? item.member_id : item.project_id;
      return itemId !== id;
    });
  }

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
                @input=${this._onInputChange}
                .value="${this.enteredValue}"
                class="input-primary peer ps-10"
                placeholder="Search ${this.type}"
                autocomplete="off"
                autocorrect="off"
                autocapitalize="off"
                spellcheck="false"
                autocomplete="off"
              />
              <div class="absolute end-1.5 top-1.5 peer-placeholder-shown:hidden">
                <button @click=${this._cleanEnteredValue} type="button" class="cursor-pointer mt-[2px]">
                  <div class="svg-icon size-5 bg-stone-400 hover:bg-stone-700 icon-close"></div>
                </button>
              </div>
              <div class="absolute z-10 start-0 end-0">
                <div
                  class="${!this.visibleDropdown
                    ? "hidden"
                    : ""} bg-white rounded-lg shadow w-full border border-stone-200 mt-1"
                >
                  ${this.visibleOptions.length > 0 && this.visibleDropdown
                    ? html`<ul class="text-sm text-stone-700 overflow-auto max-h-[180px]">
                        ${this.visibleOptions.map((option, index) => {
                          const isSelected = this.selected.some(
                            (item) => item.name === option.name && item.foundation === option.foundation,
                          );
                          return html`<li
                            class="group ${index > 0 ? "border-t border-stone-200" : ""} ${this
                              .activeIndex === index
                              ? "active"
                              : ""}"
                          >
                            <button
                              type="button"
                              @click=${() => this._onSelect(option)}
                              @mouseover=${() => (this.activeIndex = index)}
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
        ${this.selected.length > 0
          ? html` <div class="flex flex-wrap gap-5 w-full">
              ${this.selected.map(
                (opt, index) =>
                  html`<div class="relative border border-stone-200 rounded-lg p-4 pe-10 bg-white min-w-64">
                    <button
                      @click=${() => this._onRemove(this.type === "members" ? opt.member_id : opt.project_id)}
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
customElements.define("dashboard-search", DashboardSearch);
