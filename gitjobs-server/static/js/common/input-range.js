/**
 * InputRange web component for a customizable range slider with tooltip and legend.
 * Extends LitWrapper and uses Lit for rendering.
 */
import { html, createRef, ref, nothing } from "/static/vendor/js/lit-all.v3.2.1.min.js";
import { LitWrapper } from "/static/js/common/lit-wrapper.js";
import { triggerActionOnForm } from "/static/js/jobboard/filters.js";

/**
 * @class InputRange
 * @property {string} form - Form ID to trigger on change (optional).
 * @property {string} name - Name attribute for the input.
 * @property {number} min - Minimum value of the range.
 * @property {number} max - Maximum value of the range.
 * @property {number} step - Step size for the range.
 * @property {number} value - Current value of the range.
 * @property {string} prefix - Text to display before the value in the tooltip.
 * @property {string} unit - Text to display after the value in the tooltip.
 * @property {number} legendCount - Number of legend steps to display.
 * @property {boolean} visibleTooltip - Whether the tooltip is visible.
 * @property {string} colorType - Color theme key ("type-1", "type-2", "type-3").
 * @method resetRange - Resets the slider to its minimum value and hides the tooltip.
 */
export class InputRange extends LitWrapper {
  static properties = {
    form: { type: String | undefined },
    name: { type: String | undefined },
    min: { type: Number },
    max: { type: Number },
    step: { type: Number },
    value: { type: Number },
    prefix: { type: String },
    unit: { type: String },
    legendCount: { type: Number },
    visibleTooltip: { type: Boolean },
    colorType: { type: String },
  };

  /**
   * Reference to the input element.
   */
  inputRef = createRef();

  /**
   * Initializes default property values and color themes.
   */
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
    this.percentFilled = 0;
    this.thumbOffset = 0;
    this.legendCount = 5;
    this.visibleTooltip = false;
    this.legendSteps = [];
    this.colorType = "type-1";
    this.colorStyles = {
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

  /**
   * Lifecycle: Called when the component is added to the DOM.
   * Prepares legend steps and updates styles if value is set.
   */
  connectedCallback() {
    super.connectedCallback();
    this._prepareSteps();
    if (this.value > 0) {
      this._updateStyles(this.value);
    }
  }

  /**
   * Prepares the legend steps for the slider scale.
   */
  _prepareSteps() {
    this.legendSteps = this._generateRange(this.min, this.max, this.max / (this.legendCount - 1));
  }

  /**
   * Handles input change events, updates value and styles.
   * @param {Event} event
   */
  _handleInputChange(event) {
    const newValue = event.target.value;
    this.value = newValue;
    this._updateStyles(newValue);
  }

  /**
   * Updates the filled percentage and thumb offset for the slider.
   * @param {number} value
   */
  _updateStyles(value) {
    this.percentFilled = parseInt((value * 100) / this.max, 10);
    const thumbSize = 17;
    this.thumbOffset = thumbSize * (0.5 - this.percentFilled / 100);
  }

  /**
   * Sets the tooltip visibility.
   * @param {boolean} isVisible
   */
  _setTooltipVisibility(isVisible) {
    this.visibleTooltip = isVisible;
  }

  /**
   * Generates an array of numbers for the legend steps.
   * @param {number} start
   * @param {number} stop
   * @param {number} step
   * @returns {number[]}
   */
  _generateRange(start, stop, step = 1) {
    return Array(Math.ceil((stop - start) / step))
      .fill(start)
      .map((x, y) => x + y * step);
  }

  /**
   * Formats large numbers for display in the tooltip and legend.
   * @param {number} value
   * @returns {number}
   */
  _prettifyNumber(value) {
    if (value > 1000) {
      return parseInt(value / 1000);
    }
    return value;
  }

  /**
   * Handles mouse up and touch end events, hides tooltip and triggers form action.
   */
  async _onMouseUp() {
    this._setTooltipVisibility(false);

    // Wait for the update to complete
    await this.updateComplete;
    if (this.form !== "") {
      triggerActionOnForm(this.form, "submit");
    }
  }

  /**
   * Resets the slider to its minimum value and hides the tooltip.
   */
  async resetRange() {
    this.value = 0;
    this.percentFilled = 0;
    this.thumbOffset = 0;
    this.visibleTooltip = false;
    const input = this.inputRef.value;
    if (input) {
      input.value = 0;
    }

    // Wait for the update to complete
    await this.updateComplete;
  }

  /**
   * Renders the range input, tooltip, and legend.
   * @returns {import("lit").TemplateResult}
   */
  render() {
    return html`
      <div class="relative">
        <input
          ${ref(this.inputRef)}
          form="${this.form || nothing}"
          name="${this.name}"
          type="range"
          @input=${this._handleInputChange}
          @mousedown=${() => this._setTooltipVisibility(true)}
          @mouseup=${this._onMouseUp}
          @touchstart=${() => this._setTooltipVisibility(true)}
          @touchend=${this._onMouseUp}
          min="${this.min}"
          max="${this.max}"
          step="${this.step}"
          value="${this.value}"
          aria-label="${this.name}"
          class="w-full h-2 bg-stone-200 rounded-lg appearance-none cursor-pointer ${this.colorStyles[
            this.colorType
          ].thumb}"
          style="background-image: linear-gradient(90deg, ${this.colorStyles[this.colorType][
            "progress-line"
          ]} 0%, ${this.colorStyles[this.colorType]["progress-line"]} ${this
            .percentFilled}%, rgb(231 229 228 / var(--tw-bg-opacity, 1)) ${this
            .percentFilled}%, rgb(231 229 228 / var(--tw-bg-opacity, 1)) 100%);"
        />
        <div
          role="tooltip"
          aria-hidden="${!this.visibleTooltip}"
          class="duration-100 transition-opacity ${this.visibleTooltip
            ? ""
            : "opacity-0"} absolute z-10 inline-block px-2 py-1 text-sm font-medium text-white text-center ${this
            .colorStyles[this.colorType][
            "bg-color"
          ]} rounded-lg shadow-xs tooltip top-8 start-[8.5px] -ms-8 w-16"
          style="left: calc(${this.percentFilled}% + ${this.thumbOffset}px);"
        >
          <small>${this.prefix}</small><span>${this._prettifyNumber(this.value)}</span
          ><small>${this.unit}</small>
          <div
            class="h-0 w-0 border-x-[6px] border-x-transparent border-b-[6px] ${this.colorStyles[
              this.colorType
            ].peak} absolute -top-1.5 start-[calc(50%-6px)]"
          ></div>
        </div>
        <div class="mx-[15px]">
          <ul class="flex justify-between w-full h-5">
            ${this.legendSteps.map(
              (i) =>
                html`<li class="flex justify-center relative text-xs text-stone-500">
                  <span class="absolute -start-[10px]">${this._prettifyNumber(i)}</span>
                </li>`,
            )}
            <li class="flex justify-center relative text-xs text-stone-500">
              <span class="absolute -start-[15px]">${this._prettifyNumber(this.max)}${this.unit}</span>
            </li>
          </ul>
        </div>
      </div>
    `;
  }
}

/**
 * Registers the InputRange component as a custom element.
 */
customElements.define("input-range", InputRange);
