import { LitElement, html, repeat } from "https://cdn.jsdelivr.net/gh/lit/dist@3/all/lit-all.min.js";
import { unnormalize } from "/static/js/common/common.js";
import { triggerChangeOnForm } from "/static/js/jobboard/filters.js";

export class CollapsibleFilter extends LitElement {
  static properties = {
    title: { type: String },
    name: { type: String },
    options: { type: Array },
    formattedOptions: { type: Array },
    selected: { type: Array },
    maxVisibleItems: { type: Number },
    isCollapsed: { type: Boolean },
    viewType: { type: String },
    multipleSelection: { type: Boolean },
    visibleOptions: { type: Array },
    form: { type: String },
  };

  constructor() {
    super();
    this.title = "";
    this.name = "name";
    this.options = [];
    this.formattedOptions = [];
    this.selected = [];
    this.maxVisibleItems = 5;
    this.isCollapsed = true;
    this.viewType = "cols";
    this.visibleOptions = [];
    this.multipleSelection = true;
    this.form = "";
  }

  cleanSelected() {
    this.selected = [];
    this._filterOptions();
  }

  createRenderRoot() {
    // Disable shadow dom to use Tailwind CSS
    return this;
  }

  connectedCallback() {
    super.connectedCallback();
    this._checkMaxVisibleItems();
    this._filterOptions();
  }

  _checkMaxVisibleItems() {
    if (this.selected.length > this.maxVisibleItems) {
      this.maxVisibleItems = this.selected.length;
    }
  }

  _filterOptions() {
    const sortedOptions = this._sortOptions();
    if (this.isCollapsed) {
      this.visibleOptions = sortedOptions.slice(0, this.maxVisibleItems);
    } else {
      this.visibleOptions = sortedOptions;
    }
  }

  // Sort the options based on the selected order
  _sortOptions() {
    if (this.selected.length === 0) {
      return this.options;
    } else {
      const selectedOptions = [];
      const noSelectedOptions = [];
      this.options.map((opt) => {
        if (this.selected.includes(opt)) {
          selectedOptions.push(opt);
        } else {
          noSelectedOptions.push(opt);
        }
      });

      return selectedOptions.concat(noSelectedOptions);
    }
  }

  _changeCollapseState() {
    this.isCollapsed = !this.isCollapsed;
    this._filterOptions();
  }

  async _onSelect(value) {
    if (this.multipleSelection) {
      if (!this.selected.includes(value)) {
        this.selected = [...this.selected, value];
      } else {
        this.selected = this.selected.filter((item) => item !== value);
      }
    } else {
      this.selected = [value];
    }
    this._checkMaxVisibleItems();
    this._filterOptions();

    // Wait for the update to complete
    await this.updateComplete;

    // Trigger change event on the form
    triggerChangeOnForm(this.form);
  }

  render() {
    const canCollapse = this.options.length > this.maxVisibleItems;

    return html`<div class="px-6 py-7 pt-5 border-b border-gray-100">
      <div class="flex justify-between items-center">
        <div class="font-semibold text-black text-[0.8rem]/6">${this.title}</div>
        <div>
          ${canCollapse
            ? html`<button
                type="button"
                @click=${this._changeCollapseState}
                class="border hover:bg-primary-600 hover:border-primary-600 focus:ring-0 focus:outline-none focus:ring-gray-300 font-medium rounded-full text-sm p-1 text-center inline-flex items-center group"
              >
                ${this.isCollapsed
                  ? html`<div
                      class="svg-icon h-3 w-3 bg-gray-500 group-hover:bg-white icon-caret_down"
                    ></div>`
                  : html`<div class="svg-icon h-3 w-3 bg-gray-500 group-hover:bg-white icon-caret_up"></div>`}
              </button>`
            : None}
        </div>
      </div>
      <ul class="flex w-full gap-2 mt-3 ${this.viewType === "rows" ? "flex-col" : "flex-wrap"}">
        ${repeat(
          this.visibleOptions,
          (opt) => opt,
          (opt) => html`<li>
            <button
              type="button"
              @click=${() => this._onSelect(opt)}
              class="inline-flex items-center justify-between w-full px-2 py-1 bg-white border rounded-lg cursor-pointer select-none ${this.selected.includes(
                opt,
              )
                ? "border-primary-500 text-primary-500"
                : "text-gray-500 hover:text-gray-600 hover:bg-gray-50"}"
            >
              <div class="text-[0.8rem] text-center text-nowrap capitalize">${unnormalize(opt)}</div>
            </button>
            <input
              type="checkbox"
              class="hidden"
              name="${this.name}${this.multipleSelection ? "[]" : ""}"
              value="${opt}"
              ?checked=${this.selected.includes(opt)}
            />
          </li>`,
        )}
      </ul>
      ${canCollapse
        ? html`<div class="mt-4 -mb-1.5">
            <button
              data-label="{{ label }}"
              type="button"
              @click=${this._changeCollapseState}
              class="text-gray-500 hover:text-gray-700 focus:ring-0 focus:outline-none focus:ring-gray-300 font-medium text-xs"
            >
              ${this.isCollapsed ? "+ Show more" : "- Show less"}
            </button>
          </div>`
        : None}
    </div>`;
  }
}
customElements.define("collapsible-filter", CollapsibleFilter);
