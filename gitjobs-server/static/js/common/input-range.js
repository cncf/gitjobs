import { html } from "https://cdn.jsdelivr.net/gh/lit/dist@3/all/lit-all.min.js";
import { LitWrapper } from "/static/js/common/litWrapper.js";

export class InputRange extends LitWrapper {
  static properties = {
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
    this.name = undefined;
    this.min = 0;
    this.max = 100;
    this.step = 1;
    this.value = 0;
    this.prefix = "";
    this.unit = "%";
    this.percentValue = 0;
    this.tooltipValue = 0;
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
    this.tooltipValue = parseInt((value * this.max) / 100);
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

  render() {
    return html`
      <div class="relative">
        <input
          name="${this.name}"
          type="range"
          @input=${this._onInputChange}
          @mousedown=${() => this._updateTooltipVisibility(true)}
          @mouseup=${() => this._updateTooltipVisibility(false)}
          min="${this.min}"
          max="${this.max}"
          step="${this.step}"
          value="${this.value}"
          class="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer accent-primary-900"
          style="background-image: linear-gradient(90deg, var(--primary-color) 0%, var(--primary-color) ${this
            .percentValue}%, rgb(229 231 235 / var(--tw-bg-opacity, 1)) ${this
            .percentValue}%, rgb(229 231 235 / var(--tw-bg-opacity, 1)) 100%);"
        />
        <div
          role="tooltip"
          class="duration-100 transition-opacity ${this.visibleTooltip
            ? ""
            : "opacity-0"} absolute z-10 inline-block px-2 py-1 text-sm font-medium text-white text-center bg-primary-900 rounded-lg shadow-xs tooltip top-8 start-[8.5px] -ms-8 w-16"
          style="left: calc(${this.percentValue}% + ${this.offset}px);"
        >
          <small>${this.prefix}</small><span>${this.tooltipValue}</span><small>${this.unit}</small>
          <div
            class="h-0 w-0 border-x-[6px] border-x-transparent border-b-[6px] border-b-primary-900 absolute -top-1.5 start-[25px]"
          ></div>
        </div>
        <ul class="flex justify-between w-full px-[10px] h-5">
          ${this.steps.map(
            (i) => html`<li class="flex justify-center relative text-xs text-gray-500">
              <span class="absolute">${i}</span>
            </li>`,
          )}
          <li class="flex justify-center relative text-xs text-gray-500">
            <span class="absolute">${this.max}${this.unit}</span>
          </li>
        </ul>
      </div>
    `;
  }
}
customElements.define("input-range", InputRange);
