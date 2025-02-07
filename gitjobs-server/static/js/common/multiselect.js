import { LitElement, html, css } from 'https://cdn.jsdelivr.net/gh/lit/dist@3/core/lit-core.min.js';

export class MultiSelect extends LitElement {
  static styles = css`p { color: blue }`;

  static properties = {
    id: {type: String},
    name: {type: String},
    options: {type: Array},
    selected: {type: Array},
    enteredValue: {type: String},
    visibleOptions: {type: Array},
    visibleDropdown: {type: Boolean},
  };

  constructor() {
    super();
    this.id = 'id';
    this.name = 'name';
    this.options = [];
    this.selected = [];
    this.enteredValue = '';
    this.visibleOptions = [];
    this.visibleDropdown = false;
  }

  connectedCallback() {
    super.connectedCallback()
    this.filterOptions();
  }

  createRenderRoot() {
    // Disable shadow dom to use Tailwind CSS
    return this;
  }

  filterOptions() {
    if (this.enteredValue.length > 0) {
      this.visibleOptions = this.options.filter((option) => option.toLowerCase().includes(this.enteredValue.toLowerCase()));
    } else {
      this.visibleOptions = this.options;
    }
  }

  firstUpdated(){
    window.addEventListener('mousedown', this.handleClickOutside);
  }

  handleClickOutside = (e) => {
    if(!this.contains(e.target)){
      this.visibleDropdown = false;
    }
}

  render() {

    return html`
      <label for="${this.id}" class="block text-sm/6 font-medium text-gray-900">${this.name}</label>
      <div class="mt-2 relative">
        <div class="input-primary flex flex-wrap focus-within:outline focus-within:outline-[3px] focus-within:-outline-offset-2 focus-within:outline-primary-600">
          <div class="flex flex-items flex-wrap gap-2 w-full">
            ${this.selected.map((option) => html`<span class="inline-flex items-center text-nowrap max-w-[100%] px-2 py-1 me-2 text-sm font-medium text-gray-800 bg-gray-100 rounded-sm">
              <div class="flex items-center w-full">
                <div class="truncate">${option}</div>
                <button type="button" @click=${() => this._onRemoveBadge(option)} class="inline-flex items-center p-1 ms-2 bg-transparent rounded-xs hover:bg-gray-200" aria-label="Remove badge">
                  <div class="svg-icon size-3 icon-close bg-gray-400 hover:bg-gray-900"></div>
                  <span class="sr-only">Remove badge</span>
                </button>
              </div>
            `)}
            <input type="text"
              @input=${this._onInputChange}
              @focus=${() => this.visibleDropdown = true}
              .value="${this.enteredValue}"
              placeholder="Type to search"
              class="flex flex-grow px-0 py-1 rounded-md text-gray-900 max-w-full min-w-[80px] border-0 focus:ring-0 sm:text-sm/6">
          </div>
        </div>
        <div class=${`${!this.visibleDropdown ? 'hidden' : ''} absolute start-0 z-10 bg-white divide-y divide-gray-100 rounded-lg shadow w-full border mt-1`}>
          ${this.enteredValue.length > 0 ? html`<div class="flex items-center justify-between py-2 px-4">
            <div class="truncate">${this.enteredValue}</div>
            <button
              type="button"
              @click=${() => this._onClickOption()}
              class="btn-primary">
              Add
            </button>
          </div>` : ''}
          <ul class="text-sm text-gray-700 overflow-x-auto max-h-[150px]">
            ${this.visibleOptions.map((option) => {
              const isSelected = this.selected.includes(option);
              return html`<li>
                <button @click=${() => this._onClickOption(option)} type="button" class=${`${isSelected ? 'bg-gray-100 opacity-50' : 'cursor-pointer hover:bg-gray-100'} capitalize block w-full text-left px-4 py-2`}
                ?disabled="${isSelected}">
                  <div class="flex items-center">
                    <div class="size-3 me-2">
                      ${isSelected ? html`<div class="svg-icon size-3 icon-check bg-gray-400"></div>`: ''}
                    </div>
                    <div class="truncate">${option}</div>
                  </div>
                </button>
              </li>`
            })}
          </ul>
        </div>
      </div>
      <select id="${this.id}" name="${this.id}" multiple hidden class="hidden">
        ${this.selected.map((option) => html`<option value="${option}" selected>${option}</option>`)}
      </select>
    `;
  }

  _onInputChange(event) {
    this.enteredValue = event.target.value;
    this.filterOptions();
  }

  _onRemoveBadge(option) {
    this.selected = this.selected.filter((selectedOption) => selectedOption !== option);
  }

  _onClickOption(option) {
    this.selected.push(option || this.enteredValue);
    this.enteredValue = '';
    this.visibleDropdown = false;
    this.filterOptions();
  }
}
customElements.define('multi-select', MultiSelect);
