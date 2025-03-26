import { html, nothing } from "https://cdn.jsdelivr.net/gh/lit/dist@3/all/lit-all.min.js";
import { LitWrapper } from "/static/js/common/litWrapper.js";
import { debounce } from "/static/js/common/common.js";
import { triggerActionOnForm } from "/static/js/jobboard/filters.js";

export class SearchLocation extends LitWrapper {
  static properties = {
    location_id: { type: String },
    city: { type: String },
    state: { type: String },
    country: { type: String },
    size: { type: String },
    required: { type: Boolean },
    device: { type: String },
    form: { type: String },
    value: { type: String },
    options: { type: Array | null },
    isLoading: { type: Boolean },
    withDistance: { type: Boolean },
    distance: { type: String },
    activeIndex: { type: Number | null },
  };

  constructor() {
    super();
    this.location_id = "";
    this.city = "";
    this.state = "";
    this.country = "";
    this.size = "normal";
    this.required = false;
    this.device = "desktop";
    this.value = "";
    this.form = "";
    this.options = null;
    this.withDistance = false;
    this.distance = "";
    this.isLoading = false;
    this.activeIndex = null;
    this.defaultDistance = "100000";
  }

  connectedCallback() {
    super.connectedCallback();
    window.addEventListener("mousedown", this._handleClickOutside);
    this.value = this._formatLocation(this.city, this.state, this.country);
  }

  disconnectedCallback() {
    window.removeEventListener("mousedown", this._handleClickOutside);
    super.disconnectedCallback();
  }

  async cleanLocation() {
    this.location_id = "";
    this.city = "";
    this.state = "";
    this.country = "";
    this.value = "";

    // Wait for the update to complete
    await this.updateComplete;
  }

  _formatLocation(city, state, country) {
    if (!city && !state && !country) {
      return "";
    }
    return [city, state, country].join(", ");
  }

  _handleClickOutside = (e) => {
    if (!this.contains(e.target)) {
      this.options = null;
    }
  };

  // Fetch locations
  async _fetchData(value) {
    const url = `/locations/search?ts_query=${encodeURIComponent(value)}`;
    try {
      const response = await fetch(url);
      if (!response.ok) {
        throw new Error(`Response status: ${response.status}`);
      }

      const json = await response.json();
      this.options = json;
    } catch (error) {
      // TODO - Handle error
    } finally {
      this.isLoading = false;
    }
  }

  _onInputChange(event) {
    this._isLoading = true;
    const value = event.target.value;
    if (value.length > 2) {
      debounce(this._fetchData(value), 300);
    }
  }

  _highlightItem(direction) {
    if (this.options && this.options.length > 0) {
      if (this.activeIndex === null) {
        this.activeIndex = direction === "down" ? 0 : this.options.length - 1;
      } else {
        let newIndex = direction === "down" ? this.activeIndex + 1 : this.activeIndex - 1;
        if (newIndex >= this.options.length) {
          newIndex = 0;
        }
        if (newIndex < 0) {
          newIndex = this.options.length - 1;
        }
        this.activeIndex = newIndex;
      }
    }
  }

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
        if (this.activeIndex && this.options) {
          const activeItem = this.options[this.activeIndex];
          if (activeItem) {
            this._selectLocation(activeItem);
          }
        }
        break;
      default:
        break;
    }
  }

  async _selectLocation(location) {
    this.location_id = location.location_id;
    this.city = location.city;
    this.state = location.state;
    this.country = location.country;
    this.value = this._formatLocation(location.city, location.state, location.country);
    this.options = null;
    if (this.distance === "") {
      this.distance = this.defaultDistance;
    }

    // Wait for the update to complete
    await this.updateComplete;

    // Trigger change event on the form
    if (this.form !== "") {
      triggerActionOnForm(this.form, "submit");
    }
  }

  async _handleDistanceChange(event) {
    this.distance = event.target.value;

    // Wait for the update to complete
    await this.updateComplete;

    // Trigger change event on the form
    if (this.form !== "") {
      triggerActionOnForm(this.form, "submit");
    }
  }

  async _cleanInput() {
    this.location_id = "";
    this.city = "";
    this.state = "";
    this.country = "";
    this.value = "";
    this.options = null;

    // Wait for the update to complete
    await this.updateComplete;

    // Trigger change event on the form
    if (this.form !== "") {
      triggerActionOnForm(this.form, "submit");
    }
  }

  _addFormName() {
    return this.form && this.location_id !== "" ? `form="${this.form}"` : "";
  }

  _renderOptions() {
    return html` <div class="bg-white divide-y divide-stone-100 rounded-lg shadow w-full border mt-1">
      ${this.options.length === 0
        ? html`<div class="px-8 py-4 text-sm/6 text-stone-600 italic">No locations found</div>`
        : html`<ul class="py-2 text-stone-700 overflow-auto max-h-[180px]">
            ${this.options.map(
              (l, index) =>
                html` <li class="group ${this.activeIndex === index ? "active" : ""}" data-index="${index}">
                  <button
                    type="button"
                    @click=${() => this._selectLocation(l)}
                    @mouseover=${() => (this.activeIndex = index)}
                    class="btn-location flex items-center px-4 py-2 w-full hover:bg-stone-100 group-[.active]:bg-stone-100"
                  >
                    <div class="me-2">
                      <div class="svg-icon size-4 icon-location bg-stone-500"></div>
                    </div>
                    <div class="location-text truncate">
                      ${this._formatLocation(l.city, l.state, l.country)}
                    </div>
                  </button>
                </li>`,
            )}
          </ul>`}
    </div>`;
  }

  render() {
    return html`
      <div class="mt-2 relative location-container">
        <div class="absolute inset-y-0 rtl:inset-r-0 start-0 flex items-center ps-3 pointer-events-none">
          <div
            class="svg-icon bg-stone-300 ${this.size === "small"
              ? " size-3.5 icon-location"
              : "size-4 icon-search"}"
          ></div>
        </div>
        <input
          @keydown="${this._handleKeyDown}"
          @input=${this._onInputChange}
          type="text"
          .value=${this.value}
          class="input-primary peer ${this.size === "small"
            ? "py-0.5 peer px-9 rounded-lg text-xs/6 text-stone-500"
            : "px-10"}"
          placeholder="Search location"
          autocomplete="off"
          autocorrect="off"
          autocapitalize="off"
          spellcheck="false"
          ?required=${this.required}
        />
        ${this.location_id !== ""
          ? html`<input
                type="hidden"
                form=${this.form || nothing}
                name="location[location_id]"
                .value=${this.location_id}
                ?required=${this.required}
              />
              <input type="hidden" form=${this.form || nothing} name="location[city]" .value=${this.city} />
              <input type="hidden" form=${this.form || nothing} name="location[state]" .value=${this.state} />
              <input
                type="hidden"
                form=${this.form || nothing}
                name="location[country]"
                .value=${this.country}
              />`
          : ""}
        ${this.isLoading
          ? html`<div class="absolute ${this.size === "small" ? "end-7 top-0.5" : "end-10 top-1"}">
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

        <div class="absolute end-1.5 top-1.5 peer-placeholder-shown:hidden">
          <button @click=${this._cleanInput} type="button" class="mt-[2px]">
            <div
              class="svg-icon bg-stone-400 hover:bg-stone-700 icon-close ${this.size === "small"
                ? "size-4"
                : "size-5"}"
            ></div>
          </button>
        </div>
        <div class="absolute z-10 start-0 end-0 text-${this.size === "small" ? "[0.8rem]" : "sm"}">
          ${this.options !== null ? this._renderOptions() : ""}
        </div>
      </div>
      ${this.withDistance
        ? html`
            <div class="flex items-center space-x-3 mt-3">
              <div class="text-xs/6 text-stone-500">Max. distance</div>
              <div class="flex-grow">
                <select
                  form=${this.form}
                  name="max_distance"
                  @change=${this._handleDistanceChange}
                  class="select-primary py-0.5 text-[0.8rem]/6"
                  ?disabled=${this.location_id === ""}
                >
                  ${this.location_id === ""
                    ? html`<option value="" selected></option>`
                    : html`${["10000", "50000", "100000", "500000"].map((d) => {
                        return html`<option value=${d} ?selected=${this.distance === d}>
                          ${d === "" ? "" : `${d / 1000}km`}
                        </option>`;
                      })} `}
                </select>
              </div>
            </div>
          `
        : ""}
    `;
  }
}
customElements.define("search-location", SearchLocation);
