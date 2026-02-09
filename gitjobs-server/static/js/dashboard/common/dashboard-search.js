import { html, ifDefined } from "/static/vendor/js/lit-all.v3.3.1.min.js";
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
   * @property {'projects'|'members'|'certifications'} type - Search type
   * @property {Array} foundations - Available foundation options
   * @property {Array} certifications - Available certification options
   * @property {Array} selected - Currently selected items
   * @property {string} enteredValue - Current search input value
   * @property {Array} visibleOptions - Filtered suggestions
   * @property {boolean} visibleDropdown - Dropdown visibility state
   * @property {number|null} activeIndex - Active suggestion index
   * @property {string} selectedFoundation - Selected foundation filter
   * @property {boolean} isLoading - Loading state for suggestions
   */
  static properties = {
    type: { type: String },
    foundations: { type: Array },
    certifications: { type: Array },
    selected: { type: Array },
    enteredValue: { type: String },
    visibleOptions: { type: Array },
    visibleDropdown: { type: Boolean },
    activeIndex: { type: Number | null },
    selectedFoundation: { type: String },
    isLoading: { type: Boolean },
  };

  /** @type {string} Default foundation when none selected */
  defaultFoundation = "cncf";

  constructor() {
    super();
    this.type = "projects";
    this.foundations = [];
    this.certifications = [];
    this.selected = [];
    this.enteredValue = "";
    this.viewType = "cols";
    this.visibleOptions = [];
    this.visibleDropdown = false;
    this.activeIndex = null;
    this.selectedFoundation = this.defaultFoundation;
    this.isLoading = false;
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
   * Fetches projects or members from server, or filters certifications locally.
   * @private
   */
  async _getItems() {
    if (this.type === "certifications") {
      // Filter certifications locally from provided data
      this.visibleOptions = this.certifications.filter(
        (cert) =>
          cert.name.toLowerCase().includes(this.enteredValue.toLowerCase()) ||
          cert.short_name.toLowerCase().includes(this.enteredValue.toLowerCase()) ||
          (cert.provider && cert.provider.toLowerCase().includes(this.enteredValue.toLowerCase())),
      );
      this.visibleDropdown = true;
      this.isLoading = false;
      return;
    }

    // Fetch projects or members from server
    const url = `${
      this.type === "members" ? "/dashboard/members/search?member=" : "/projects/search?project="
    }${encodeURIComponent(this.enteredValue)}&foundation=${this.selectedFoundation}`;
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
      this.isLoading = false;
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
    const minLength = this.type === "certifications" ? 0 : 2;
    if (this.enteredValue.length >= minLength) {
      this.isLoading = true;
      debounce(this._getItems(this.enteredValue), 300);
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
   * Handles input focus - shows all certifications immediately for certification type.
   * @private
   */
  _onInputFocus() {
    if (this.type === "certifications") {
      // Show all certifications immediately on focus
      this.visibleOptions = this.certifications;
      this.visibleDropdown = true;
      this.activeIndex = null;
    }
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
    if (this.type === "projects" || this.type === "certifications" || this.type === "members") {
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
      let itemId;
      if (this.type === "members") {
        itemId = item.member_id;
      } else if (this.type === "certifications") {
        itemId = item.certification_id;
      } else {
        itemId = item.project_id;
      }
      return itemId !== id;
    });
  }

  render() {
    const getLabel = () => {
      switch (this.type) {
        case "members":
          return "Foundation member";
        case "certifications":
          return "Certifications";
        default:
          return "Projects";
      }
    };

    const getLegend = () => {
      switch (this.type) {
        case "certifications":
          return "Desired certifications for this position.";
        case "projects":
          return "If the job position involves contributing to any of the supported foundations projects, please list them here.";
        default:
          return "If your company is a member of any of the supported foundations please select the corresponding member entry. Jobs posted by members will be featured across the site. False membership claims may lead to the suspension of the employer and associated user accounts.";
      }
    };

    return html`<div>
        <label for="project" class="form-label">${getLabel()}</label>
        <div class="grid grid-cols-1 gap-x-6 gap-y-8 md:grid-cols-6 max-w-5xl">
          ${this.type !== "certifications"
            ? html`
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
              `
            : ""}

          <div class="${this.type === "certifications" ? "col-span-full" : "col-span-full lg:col-span-4"}">
            <div class="mt-2 relative">
              <div class="absolute top-2.5 start-0 flex items-center ps-3 pointer-events-none">
                <div class="svg-icon size-4 icon-search bg-stone-300"></div>
              </div>
              <input
                type="text"
                @keydown="${this._handleKeyDown}"
                @input=${this._onInputChange}
                @focus=${this._onInputFocus}
                .value="${this.enteredValue}"
                class="input-primary peer ps-10"
                placeholder="Search ${this.type}"
                autocomplete="off"
                autocorrect="off"
                autocapitalize="off"
                spellcheck="false"
              />
              <div class="absolute end-1.5 top-1.5 peer-placeholder-shown:hidden">
                <button @click=${this._cleanEnteredValue} type="button" class="cursor-pointer mt-[2px]">
                  <div class="svg-icon size-5 bg-stone-400 hover:bg-stone-700 icon-close"></div>
                </button>
              </div>
              ${this.isLoading
                ? html`<div class="absolute end-7 top-1">
                    <div role="status">
                      <svg
                        aria-hidden="true"
                        class="inline size-5 text-stone-200 animate-spin fill-primary-600"
                        viewBox="0 0 100 101"
                        fill="none"
                        xmlns="http://www.w3.org/2000/svg"
                      >
                        <path
                          d="M100 50.5908C100 78.2051 77.6142 100.591 50 100.591C22.3858 100.591 0 78.2051 0 50.5908C0 22.9766 22.3858 0.59082 50 0.59082C77.6142 0.59082 100 22.9766 100 50.5908ZM9.08144 50.5908C9.08144 73.1895 27.4013 91.5094 50 91.5094C72.5987 91.5094 90.9186 73.1895 90.9186 50.5908C90.9186 27.9921 72.5987 9.67226 50 9.67226C27.4013 9.67226 9.08144 27.9921 9.08144 50.5908Z"
                          fill="currentColor"
                        />
                        <path
                          d="M93.9676 39.0409C96.393 38.4038 97.8624 35.9116 97.0079 33.5539C95.2932 28.8227 92.871 24.3692 89.8167 20.348C85.8452 15.1192 80.8826 10.7238 75.2124 7.41289C69.5422 4.10194 63.2754 1.94025 56.7698 1.05124C51.7666 0.367541 46.6976 0.446843 41.7345 1.27873C39.2613 1.69328 37.813 4.19778 38.4501 6.62326C39.0873 9.04874 41.5694 10.4717 44.0505 10.1071C47.8511 9.54855 51.7191 9.52689 55.5402 10.0491C60.8642 10.7766 65.9928 12.5457 70.6331 15.2552C75.2735 17.9648 79.3347 21.5619 82.5849 25.841C84.9175 28.9121 86.7997 32.2913 88.1811 35.8758C89.083 38.2158 91.5421 39.6781 93.9676 39.0409Z"
                          fill="currentFill"
                        />
                      </svg>
                      <span class="sr-only">Loading...</span>
                    </div>
                  </div>`
                : ""}
              <div class="absolute z-10 start-0 end-0">
                <div
                  class="${!this.visibleDropdown
                    ? "hidden"
                    : ""} bg-white rounded-lg shadow w-full border border-stone-200 mt-1"
                >
                  ${this.visibleOptions.length > 0 && this.visibleDropdown
                    ? html`<ul class="text-sm text-stone-700 overflow-auto max-h-[180px]">
                        ${this.visibleOptions.map((option, index) => {
                          const isSelected = this.selected.some((item) => {
                            if (this.type === "certifications") {
                              return item.certification_id === option.certification_id;
                            }
                            return item.name === option.name && item.foundation === option.foundation;
                          });
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
                                  ${this.type === "certifications"
                                    ? html`<div class="inline-flex">
                                          <div
                                            class="truncate text-nowrap max-w-[100%] text-xs/6 font-medium text-stone-700"
                                          >
                                            ${option.short_name}
                                            <span class="font-normal text-stone-500/75"
                                              >(${option.provider})</span
                                            >
                                          </div>
                                        </div>
                                        <div class="truncate text-start text-stone-700 font-medium">
                                          ${option.name}
                                        </div>`
                                    : html`<div class="truncate text-start text-stone-700 font-medium">
                                          ${option.name}
                                        </div>
                                        <div class="inline-flex">
                                          <div
                                            class="truncate text-nowrap uppercase max-w-[100%] text-xs/6 font-medium text-stone-500/75"
                                          >
                                            ${option.foundation}
                                            ${this.type === "projects"
                                              ? option.maturity
                                              : `${option.level} member`}
                                          </div>
                                        </div>`}
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
        <p class="form-legend">${getLegend()}</p>
      </div>
      <div class="col-span-full mt-4">
        ${this.selected.length > 0
          ? html` <div class="flex flex-wrap gap-5 w-full">
              ${this.selected.map(
                (opt, index) =>
                  html`<div
                    class="relative border border-stone-200 rounded-lg p-4 pe-10 bg-white ${this.type ===
                    "certifications"
                      ? "min-w-full"
                      : "min-w-64"}"
                    title="${ifDefined(opt.description)}"
                  >
                    <button
                      @click=${() =>
                        this._onRemove(
                          this.type === "members"
                            ? opt.member_id
                            : this.type === "certifications"
                              ? opt.certification_id
                              : opt.project_id,
                        )}
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
                        ${this.type === "certifications"
                          ? html` <div class="inline-flex">
                                <div
                                  class="truncate text-nowrap max-w-[100%] text-xs/6 font-medium text-stone-700"
                                >
                                  ${opt.short_name}
                                  <span class="font-normal text-stone-500/75">(${opt.provider})</span>
                                </div>
                              </div>
                              <div class="truncate text-start text-stone-700 font-medium ">
                                ${opt.url
                                  ? html`
                                      <a
                                        href="${opt.url}"
                                        target="_blank"
                                        rel="noopener noreferrer"
                                        class="inline-flex items-baseline max-w-full hover:underline"
                                      >
                                        <span class="truncate">${opt.name}</span>
                                        <div
                                          class="svg-icon size-2 icon-external_link bg-stone-500 ms-2 srink-0"
                                        ></div>
                                      </a>
                                    `
                                  : opt.name}
                              </div>`
                          : html`<div class="truncate text-start text-stone-700 font-medium ">
                                ${opt.name}
                              </div>
                              <div class="inline-flex">
                                <div
                                  class="truncate text-nowrap uppercase max-w-[100%] text-xs/6 font-medium text-stone-500/75"
                                >
                                  ${opt.foundation}
                                  ${this.type === "members" ? `${opt.level} member` : opt.maturity}
                                </div>
                              </div>`}
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
                      : this.type === "certifications"
                        ? html`<input
                              type="hidden"
                              name="certifications[${index}][certification_id]"
                              value="${opt.certification_id}"
                            />
                            <input type="hidden" name="certifications[${index}][name]" value="${opt.name}" />
                            <input
                              type="hidden"
                              name="certifications[${index}][short_name]"
                              value="${opt.short_name}"
                            />
                            <input
                              type="hidden"
                              name="certifications[${index}][provider]"
                              value="${opt.provider}"
                            />
                            ${opt.logo_url
                              ? html`<input
                                  type="hidden"
                                  name="certifications[${index}][logo_url]"
                                  value="${opt.logo_url}"
                                />`
                              : ""}
                            ${opt.description
                              ? html`<input
                                  type="hidden"
                                  name="certifications[${index}][description]"
                                  value="${opt.description}"
                                />`
                              : ""}
                            ${opt.url
                              ? html`<input
                                  type="hidden"
                                  name="certifications[${index}][url]"
                                  value="${opt.url}"
                                />`
                              : ""}`
                        : html`<input
                              type="hidden"
                              name="members[${index}][member_id]"
                              value="${opt.member_id}"
                            />
                            <input type="hidden" name="members[${index}][name]" value="${opt.name}" />
                            <input type="hidden" name="members[${index}][level]" value="${opt.level}" />
                            <input
                              type="hidden"
                              name="members[${index}][foundation]"
                              value="${opt.foundation}"
                            />
                            <input
                              type="hidden"
                              name="members[${index}][logo_url]"
                              value="${opt.logo_url}"
                            />`}
                  </div> `,
              )}
            </div>`
          : ""}
      </div>`;
  }
}
customElements.define("dashboard-search", DashboardSearch);
