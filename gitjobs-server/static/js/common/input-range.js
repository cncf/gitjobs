import { html, createRef, ref, nothing } from "/static/vendor/js/lit-all.v3.3.1.min.js";
import { LitWrapper } from "/static/js/common/lit-wrapper.js";
import { triggerActionOnForm } from "/static/js/common/common.js";

/**
 * Custom range input component with visual feedback and legends.
 * Supports custom styling, tooltips, and form integration.
 * @extends LitWrapper
 */
export class InputRange extends LitWrapper {
  /**
   * Component properties definition
   * @property {string} form - Form ID for input association
   * @property {string} name - Input name attribute
   * @property {number} min - Minimum value
   * @property {number} max - Maximum value
   * @property {number} step - Step increment
   * @property {number} value - Current value
   * @property {string} prefix - Value prefix (e.g., '$')
   * @property {string} unit - Value unit (e.g., '%', 'k')
   * @property {number} legendsNumber - Number of legend markers
   * @property {boolean} visibleTooltip - Tooltip visibility state
   * @property {string} type - Visual style type (type-1, type-2, type-3)
   */
  static properties = {
    form: { type: String | undefined },
    name: { type: String | undefined },
    min: { type: Number },
    max: { type: Number },
    step: { type: Number },
    value: { type: Number },
    prefix: { type: String },
    unit: { type: String },
    legendsNumber: { type: Number },
    visibleTooltip: { type: Boolean },
    type: { type: String },
  };

  /** @type {import('lit').Ref<HTMLInputElement>} Reference to input element */
  inputRef = createRef();

  constructor() {
    super();
    this.form = undefined;
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
    this.legendSteps = [];
    this.type = "type-1";
    this.colors = {
      "type-1": {
        "progress-line": "var(--color-primary-500)",
        thumb: "accent-primary-600",
        "bg-color": "bg-primary-900",
        peak: "border-b-primary-900",
      },
      "type-2": {
        "progress-line": "var(--color-lime-500)",
        thumb: "accent-lime-600",
        "bg-color": "bg-lime-900",
        peak: "border-b-lime-900",
      },
      "type-3": {
        "progress-line": "var(--color-lime-300)",
        thumb: "accent-lime-400",
        "bg-color": "bg-lime-800",
        peak: "border-b-lime-800",
      },
    };
  }

  connectedCallback() {
    super.connectedCallback();

    this.legendSteps = this._range(this.min, this.max, this.max / (this.legendsNumber - 1));

    if (this.value > 0) {
      this._refreshStyles(this.value);
    }
  }

  /**
   * Handles input value changes and updates visual feedback.
   * @param {Event} event - Input change event
   * @private
   */
  _onInputChange(event) {
    const value = event.target.value;
    this.value = value;
    this._refreshStyles(value);
  }

  /**
   * Updates the visual styles based on the current value.
   * Calculates percentage and thumb offset for accurate positioning.
   * @param {number} value - The current range value
   * @private
   */
  _refreshStyles(value) {
    this.percentValue = Math.round((value * 100) / this.max);
    const thumbSize = 17;
    this.offset = thumbSize * (0.5 - this.percentValue / 100);
  }

  /**
   * Updates tooltip visibility state.
   * @param {boolean} status - True to show, false to hide
   * @private
   */
  _updateTooltipVisibility(status) {
    this.visibleTooltip = status;
  }

  /**
   * Generates an array of numbers within a range.
   * @param {number} start - Start value
   * @param {number} stop - End value (exclusive)
   * @param {number} [step=1] - Step increment
   * @returns {number[]} Array of numbers in range
   * @private
   */
  _range(start, stop, step = 1) {
    return Array(Math.ceil((stop - start) / step))
      .fill(start)
      .map((x, y) => x + y * step);
  }

  /**
   * Formats large numbers for display (e.g., 1000 -> 1).
   * @param {number} value - The number to format
   * @returns {number} Formatted number
   * @private
   */
  _prettyNumber(value) {
    if (value > 1000) {
      return Math.trunc(value / 1000);
    }
    return value;
  }

  /**
   * Handles mouse/touch release events.
   * Hides tooltip and triggers form submission if configured.
   * @private
   */
  async _mouseup() {
    this._updateTooltipVisibility(false);

    // Wait for the update to complete
    await this.updateComplete;

    // Trigger change event on the form
    if (this.form !== "") {
      triggerActionOnForm(this.form, "submit");
    }
  }

  /**
   * Public method to reset the range input to its initial state.
   */
  async cleanRange() {
    this.value = 0;
    this.percentValue = 0;
    this.offset = 0;
    this.visibleTooltip = false;
    const input = this.inputRef.value;
    if (input) {
      input.value = 0;
    }

    // Wait for the update to complete
    await this.updateComplete;
  }

  render() {
    return html`
      <div class="relative">
        <input
          ${ref(this.inputRef)}
          form="${this.form || nothing}"
          name="${this.name}"
          type="range"
          @input=${this._onInputChange}
          @mousedown=${() => this._updateTooltipVisibility(true)}
          @mouseup=${this._mouseup}
          @touchstart=${() => this._updateTooltipVisibility(true)}
          @touchend=${this._mouseup}
          min="${this.min}"
          max="${this.max}"
          step="${this.step}"
          value="${this.value}"
          class="w-full h-2 bg-stone-200 rounded-lg appearance-none cursor-pointer ${this.colors[this.type]
            .thumb}"
          style="background-image: linear-gradient(90deg, ${this.colors[this.type][
            "progress-line"
          ]} 0%, ${this.colors[this.type]["progress-line"]} ${this
            .percentValue}%, rgb(231 229 228 / var(--tw-bg-opacity, 1)) ${this
            .percentValue}%, rgb(231 229 228 / var(--tw-bg-opacity, 1)) 100%);"
        />
        <div
          role="tooltip"
          class="duration-100 transition-opacity ${this.visibleTooltip
            ? ""
            : "opacity-0"} absolute z-10 inline-block px-2 py-1 text-sm font-medium text-white text-center ${this
            .colors[this.type]["bg-color"]} rounded-lg shadow-xs tooltip top-8 start-[8.5px] -ms-8 w-16"
          style="left: calc(${this.percentValue}% + ${this.offset}px);"
        >
          <small>${this.prefix}</small><span>${this._prettyNumber(this.value)}</span
          ><small>${this.unit}</small>
          <div
            class="h-0 w-0 border-x-[6px] border-x-transparent border-b-[6px] ${this.colors[this.type]
              .peak} absolute -top-1.5 start-[calc(50%-6px)]"
          ></div>
        </div>
        <div class="mx-[15px]">
          <ul class="flex justify-between w-full h-5">
            ${this.legendSteps.map(
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
