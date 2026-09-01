import { expect } from "@open-wc/testing";

import "/static/js/common/input-range.js";
import { waitForMicrotask } from "/tests/unit/test-utils/async.js";
import { mockHtmx } from "/tests/unit/test-utils/globals.js";
import { mountLitComponent, useMountedElementsCleanup } from "/tests/unit/test-utils/lit.js";

describe("input-range", () => {
  let htmx;

  useMountedElementsCleanup("input-range");

  beforeEach(() => {
    htmx = mockHtmx();
  });

  afterEach(() => {
    htmx.restore();
  });

  const renderRange = async (properties = {}) => {
    document.body.innerHTML = '<form id="salary-form"></form>';
    return mountLitComponent("input-range", {
      form: "salary-form",
      max: 100,
      min: 0,
      name: "salary",
      step: 5,
      value: 25,
      ...properties,
    });
  };

  it("renders a native range input in light DOM", async () => {
    // Render the range component with its associated form contract.
    const element = await renderRange();
    const input = element.querySelector('input[type="range"]');

    // Verify light-DOM rendering and native input attributes.
    expect(element.shadowRoot).to.equal(null);
    expect(input).to.not.equal(null);
    expect(input.getAttribute("form")).to.equal("salary-form");
    expect(input.name).to.equal("salary");
    expect(input.min).to.equal("0");
    expect(input.max).to.equal("100");
    expect(input.step).to.equal("5");
    expect(input.value).to.equal("25");
  });

  it("submits the native input name and value", async () => {
    // Render the range control associated with the fixture form.
    await renderRange();

    // Serialize the real form payload exposed by the custom element.
    const formData = new FormData(document.getElementById("salary-form"));

    // Verify the native input contributes its public name and value.
    expect(formData.get("salary")).to.equal("25");
  });

  it("updates value, progress, and tooltip text from user input", async () => {
    // Render the range component with formatted tooltip content.
    const element = await renderRange({ prefix: "$", unit: "k" });
    const input = element.querySelector('input[type="range"]');

    // Apply a new value through the native input event.
    input.value = "60";
    input.dispatchEvent(new Event("input", { bubbles: true }));
    await element.updateComplete;

    // Verify submitted, visual, and tooltip state update together.
    expect(String(element.value)).to.equal("60");
    expect(element.percentValue).to.equal(60);
    expect(element.getAttribute("style")).to.equal(null);
    expect(element.querySelector('[role="tooltip"]').textContent.replace(/\s+/g, "")).to.equal("$60k");
    expect(input.getAttribute("style")).to.contain("60%");
  });

  it("shows the tooltip during interaction and submits on release", async () => {
    // Render the range control used for pointer interaction.
    const element = await renderRange();
    const input = element.querySelector('input[type="range"]');

    // Begin interaction and verify the tooltip becomes visible.
    input.dispatchEvent(new MouseEvent("mousedown", { bubbles: true }));
    await element.updateComplete;
    expect(element.querySelector('[role="tooltip"]').classList.contains("opacity-0")).to.equal(false);

    // Release the control and allow its async submission path to complete.
    input.dispatchEvent(new MouseEvent("mouseup", { bubbles: true }));
    await element.updateComplete;
    await waitForMicrotask();

    // Verify release hides the tooltip and submits the associated form.
    expect(element.querySelector('[role="tooltip"]').classList.contains("opacity-0")).to.equal(true);
    expect(htmx.triggerCalls).to.deep.equal([[document.getElementById("salary-form"), "submit"]]);
  });

  it("resets submitted and visible state", async () => {
    // Render the range control with a non-zero initial value.
    const element = await renderRange({ value: 70 });

    // Reset the component through its public cleanup method.
    await element.cleanRange();

    // Read the rendered input and serialized form state after cleanup.
    const input = element.querySelector('input[type="range"]');
    const formData = new FormData(document.getElementById("salary-form"));

    // Verify component, input, tooltip, and form values reset together.
    expect(element.value).to.equal(0);
    expect(element.percentValue).to.equal(0);
    expect(element.visibleTooltip).to.equal(false);
    expect(input.value).to.equal("0");
    expect(formData.get("salary")).to.equal("0");
  });

  it("does not duplicate rendered controls after reconnecting", async () => {
    // Render and reconnect the light-DOM component.
    const element = await renderRange();

    element.remove();
    document.body.append(element);
    await element.updateComplete;

    // Verify reconnecting preserves one input and one tooltip.
    expect(element.querySelectorAll('input[type="range"]')).to.have.length(1);
    expect(element.querySelectorAll('[role="tooltip"]')).to.have.length(1);
  });
});
