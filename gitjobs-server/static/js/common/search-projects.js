import { html } from "/static/vendor/js/lit-all.v3.2.1.min.js";
import { unnormalize } from "/static/js/common/common.js";
import { triggerActionOnForm } from "/static/js/jobboard/filters.js";
import { LitWrapper } from "/static/js/common/lit-wrapper.js";
import { debounce } from "/static/js/common/common.js";

/**
 * SearchProjects web component for searching and selecting projects by foundation.
 * Extends LitWrapper and uses Lit for rendering.
 *
 * @class SearchProjects
 * @property {Array} foundations - List of available foundations.
 * @property {Array} selectedProjects - Array of selected project objects.
 * @property {String} inputValue - Current value of the search input field.
 * @property {Array|null} visibleOptions - Project options shown in the dropdown.
 * @property {Boolean} isDropdownOpen - Dropdown visibility state.
 * @property {String} formId - Associated form ID.
 * @property {Number|null} highlightedIndex - Highlighted dropdown index.
 * @property {String|null} selectedFoundation - Selected foundation name.
 */
export class SearchProjects extends LitWrapper {
  static properties = {
    foundations: { type: Array },
    selectedProjects: { type: Array },
    inputValue: { type: String },
    visibleOptions: { type: Array | null },
    isDropdownOpen: { type: Boolean },
    formId: { type: String },
    highlightedIndex: { type: Number | null },
    selectedFoundation: { type: String | null },
  };

  /**
   * Initializes component state.
   */
  constructor() {
    super();
    this.foundations = [];
    this.selectedProjects = [];
    this.inputValue = "";
    this.visibleOptions = null;
    this.isDropdownOpen = false;
    this.formId = "";
    this.highlightedIndex = null;
    this.selectedFoundation = null;
  }

  /**
   * Adds event listener for outside clicks when component is attached.
   */
  connectedCallback() {
    super.connectedCallback();
    window.addEventListener("mousedown", this._handleOutsideClick);
  }

  /**
   * Removes event listener for outside clicks when component is detached.
   */
  disconnectedCallback() {
    super.disconnectedCallback();
    window.removeEventListener("mousedown", this._handleOutsideClick);
  }

  /**
   * Clears all selected projects and resets the selected foundation.
   * Awaits update completion.
   */
  async clearSelectedProjects() {
    this.selectedProjects = [];
    this.selectedFoundation = null;
    await this.updateComplete;
  }

  /**
   * Fetches project options from the server based on input and foundation.
   * Updates visibleOptions and shows dropdown.
   */
  async _fetchProjects() {
    const url = `/projects/search?project=${encodeURIComponent(this.inputValue)}&foundation=${this.selectedFoundation}`;
    try {
      const response = await fetch(url);
      if (!response.ok) {
        throw new Error(`Response status: ${response.status}`);
      }
      const json = await response.json();
      this.visibleOptions = json;
    } catch (error) {
      // TODO - Handle error
    } finally {
      this.isDropdownOpen = true;
    }
  }

  /**
   * Handles foundation dropdown change event.
   * Resets input and options when foundation changes.
   * @param {Event} event - The change event.
   */
  _handleFoundationChange(event) {
    const selectedFoundation = event.target.value;
    if (selectedFoundation === "") {
      this.selectedFoundation = null;
    } else {
      this.selectedFoundation = selectedFoundation;
    }
    this.visibleOptions = null;
    this.inputValue = "";
    this.isDropdownOpen = false;
  }

  /**
   * Filters project options based on input length.
   * Triggers debounced fetch if input is long enough.
   */
  _filterProjectOptions() {
    if (this.inputValue.length > 2) {
      debounce(this._fetchProjects(this.inputValue), 300);
    } else {
      this.visibleOptions = null;
      this.isDropdownOpen = false;
      this.highlightedIndex = null;
    }
  }

  /**
   * Handles input change event for the search field.
   * @param {Event} event - The input event.
   */
  _handleInputChange(event) {
    this.inputValue = event.target.value;
    this._filterProjectOptions();
  }

  /**
   * Clears the search input and hides the dropdown.
   */
  _cleanInputValue() {
    this.inputValue = "";
    this.isDropdownOpen = false;
    this.visibleOptions = null;
    this.highlightedIndex = null;
  }

  /**
   * Handles clicks outside the component to close the dropdown.
   * @param {MouseEvent} e - The mouse event.
   */
  _handleOutsideClick = (e) => {
    if (!this.contains(e.target)) {
      this._cleanInputValue();
    }
  };

  /**
   * Handles keyboard navigation and selection in the dropdown.
   * Supports ArrowDown, ArrowUp, and Enter keys.
   * @param {KeyboardEvent} event - The keyboard event.
   */
  _handleKeyDown(event) {
    switch (event.key) {
      // Highlight the next item in the list
      case "ArrowDown":
        this._highlightProjectOption("down");
        break;
      // Highlight the previous item in the list
      case "ArrowUp":
        this._highlightProjectOption("up");
        break;
      // Select the highlighted item
      case "Enter":
        event.preventDefault();
        if (
          this.highlightedIndex !== null &&
          this.visibleOptions !== null &&
          this.visibleOptions.length > 0
        ) {
          const activeItem = this.visibleOptions[this.highlightedIndex];
          if (activeItem) {
            this._selectProject(activeItem);
          }
        }
        break;
      default:
        break;
    }
  }

  /**
   * Highlights the next or previous project option in the dropdown.
   * Wraps around if at the end or beginning.
   * @param {"up"|"down"} direction - Direction to move highlight.
   */
  _highlightProjectOption(direction) {
    if (this.visibleOptions && this.visibleOptions.length > 0) {
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
   * Adds a project to the selectedProjects array and resets input.
   * Triggers form action after update.
   * @param {Object} value - The selected project object.
   */
  async _selectProject(value) {
    this.selectedProjects.push(value);
    this.inputValue = "";
    this.isDropdownOpen = false;
    this.visibleOptions = null;
    this.highlightedIndex = null;

    // Remove selected foundation on select out of this component
    const foundationSelects = document.getElementsByName("foundation");
    foundationSelects.forEach((select) => {
      if (select.value !== "") {
        select.value = "";
      }
    });

    // Wait for the update to complete
    await this.updateComplete;

    // Trigger change event on the form
    triggerActionOnForm(this.formId, "submit");
  }

  /**
   * Removes a project from the selectedProjects array by name.
   * Triggers form action after update.
   * @param {string} name - The name of the project to remove.
   */
  async _removeProject(name) {
    this.selectedProjects = this.selectedProjects.filter((item) => item.name !== name);

    // Wait for the update to complete
    await this.updateComplete;

    // Trigger change event on the form
    triggerActionOnForm(this.formId, "submit");
  }

  /**
   * Renders the component UI: foundation select, search input, dropdown, and selected projects.
   * @returns {TemplateResult} The rendered HTML template.
   */
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
          @input=${this._handleInputChange}
          type="text"
          .value="${this.inputValue}"
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
          <button @click=${this._cleanInputValue} type="button" class="cursor-pointer mt-[2px]">
            <div class="svg-icon size-5 bg-stone-400 hover:bg-stone-700 icon-close"></div>
          </button>
        </div>
        <div class="absolute z-10 start-0 end-0">
          <div
            class="${!this.isDropdownOpen
              ? "hidden"
              : ""} bg-white divide-y divide-stone-100 rounded-lg shadow w-full border border-stone-200 mt-1"
          >
            ${this.visibleOptions !== null && this.visibleOptions.length > 0 && this.isDropdownOpen
              ? html`<ul class="text-sm text-stone-700 overflow-auto max-h-[180px]">
                  ${this.visibleOptions.map((option, index) => {
                    const isSelected = this.selectedProjects.some(
                      (item) => item.name === option.name && item.foundation === option.foundation,
                    );
                    return html`<li
                      class="group ${this.highlightedIndex === index ? "active" : ""}"
                      data-index="${index}"
                    >
                      <button
                        type="button"
                        @click=${() => this._selectProject(option)}
                        @mouseover=${() => (this.highlightedIndex = index)}
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
        ${this.selectedProjects.length > 0
          ? html`<div class="flex gap-2 mt-4 flex-col">
              ${this.selectedProjects.map(
                (opt, index) =>
                  html` <button
                      type="button"
                      @click=${() => this._removeProject(opt.name)}
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
                      form="${this.formId}"
                      name="projects[${index}][name]"
                      value="${opt.name}"
                    />
                    <input
                      type="hidden"
                      form="${this.formId}"
                      name="projects[${index}][foundation]"
                      value="${opt.foundation}"
                    />`,
              )}
            </div>`
          : ""}
      </div>`;
  }
}

/**
 * Registers the SearchProjects component as a custom element.
 */
customElements.define("search-projects", SearchProjects);
