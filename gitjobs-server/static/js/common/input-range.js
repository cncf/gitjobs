import { html } from "https://cdn.jsdelivr.net/gh/lit/dist@3/all/lit-all.min.js";
import { LitWrapper } from "/static/js/common/litWrapper.js";
import { triggerActionOnForm } from "/static/js/jobboard/filters.js";

export class InputRange extends LitWrapper {
  static properties = {
    form: { type: String },
    name: { type: String },
    min: { type: Number },
    max: { type: Number },
    step: { type: Number },
    value: { type: Number },
    prefix: { type: String },
    unit: { type: String },
    legendsNumber: { type: Number },
    visibleTooltip: { type: Boolean },
  };

  constructor() {
    super();
    this.form = "";
    this.name = undefined;
    this.min = 0;
    this.max = 100;
    this.step = 1;
    this.value = 0;
    this.prefix = "";
    this.unit = "%";
    this.percentValue = 0;
    this.offset = 0;
    this.legendsNumber = 5;
    this.visibleTooltip = false;
    this.steps = [];
  }

  connectedCallback() {
    super.connectedCallback();

    this.steps = this._range(this.min, this.max, this.max / (this.legendsNumber - 1));

    if (this.value > 0) {
      this._refreshStyles(this.value);
    }
  }

  _onInputChange(event) {
    const value = event.target.value;
    this.value = value;
    this._refreshStyles(value);
  }

  _refreshStyles(value) {
    this.percentValue = parseInt((value * 100) / this.max, 10);
    const thumbSize = 17;
    this.offset = thumbSize * (0.5 - this.percentValue / 100);
  }

  _updateTooltipVisibility(status) {
    this.visibleTooltip = status;
  }

  _range(start, stop, step = 1) {
    return Array(Math.ceil((stop - start) / step))
      .fill(start)
      .map((x, y) => x + y * step);
  }

  _prettyNumber(value) {
    if (value > 1000) {
      return parseInt(value / 1000);
    }
    return value;
  }

  async _mouseup() {
    this._updateTooltipVisibility(false);

    // Wait for the update to complete
    await this.updateComplete;

    // Trigger change event on the form
    if (this.form !== "") {
      triggerActionOnForm(this.form, "submit");
    }
  }

  async cleanRange() {
    this.value = 0;
    this.percentValue = 0;
    this.offset = 0;
    this.visibleTooltip = false;

    // Wait for the update to complete
    await this.updateComplete;
  }

  render() {
    return html`
      <div class="relative">
        <input
          form="${this.form}"
          name="${this.name}"
          type="range"
          @input=${this._onInputChange}
          @mousedown=${() => this._updateTooltipVisibility(true)}
          @mouseup=${this._mouseup}
          min="${this.min}"
          max="${this.max}"
          step="${this.step}"
          value="${this.value}"
          class="w-full h-2 bg-stone-200 rounded-lg appearance-none cursor-pointer accent-primary-300"
          style="background-image: linear-gradient(90deg, var(--primary-color) 0%, var(--primary-color) ${this
            .percentValue}%, rgb(231 229 228 / var(--tw-bg-opacity, 1)) ${this
            .percentValue}%, rgb(231 229 228 / var(--tw-bg-opacity, 1)) 100%);"
        />
        <div
          role="tooltip"
          class="duration-100 transition-opacity ${this.visibleTooltip
            ? ""
            : "opacity-0"} absolute z-10 inline-block px-2 py-1 text-sm font-medium text-white text-center bg-primary-900 rounded-lg shadow-xs tooltip top-8 start-[8.5px] -ms-8 w-16"
          style="left: calc(${this.percentValue}% + ${this.offset}px);"
        >
          <small>${this.prefix}</small><span>${this._prettyNumber(this.value)}</span
          ><small>${this.unit}</small>
          <div
            class="h-0 w-0 border-x-[6px] border-x-transparent border-b-[6px] border-b-primary-900 absolute -top-1.5 start-[25px]"
          ></div>
        </div>
        <div class="mx-[15px]">
          <ul class="flex justify-between w-full h-5">
            ${this.steps.map(
              (i) =>
                html`<li class="flex justify-center relative text-xs text-stone-500">
                  <span class="absolute -start-[10px]">${this._prettyNumber(i)}</span>
                </li>`,
            )}
            <li class="flex justify-center relative text-xs text-stone-500">
              <span class="absolute -start-[15px]">${this._prettyNumber(this.max)}${this.unit}</span>
            </li>
          </ul>
        </div>
      </div>
    `;
  }
}
customElements.define("input-range", InputRange);
