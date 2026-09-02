import { expect } from "@open-wc/testing";

import {
  addParamToQueryString,
  debounce,
  ensureElementId,
  prettifyNumber,
  setDrawerVisibility,
  triggerActionOnForm,
  unnormalize,
} from "/static/js/common/common.js";
import { waitForMicrotask } from "/tests/unit/test-utils/async.js";
import { resetDom, setLocationPath } from "/tests/unit/test-utils/dom.js";
import { mockHtmx } from "/tests/unit/test-utils/globals.js";

describe("common browser helpers", () => {
  let htmx;

  beforeEach(() => {
    resetDom();
    setLocationPath("/");
    htmx = mockHtmx();
    delete document.__unitElementIdCounter;
  });

  afterEach(() => {
    htmx.restore();
    resetDom();
    setLocationPath("/");
    delete document.__unitElementIdCounter;
  });

  it("debounces repeated calls with the latest arguments", async () => {
    // Capture values delivered by the debounced callback.
    const calls = [];
    const debounced = debounce((...args) => calls.push(args), 0);

    // Schedule two calls before the debounce delay completes.
    debounced("first");
    debounced("second", 2);
    await waitForMicrotask();

    // Verify only the latest arguments reach the callback.
    expect(calls).to.deep.equal([["second", 2]]);
  });

  it("cancels pending debounced calls", async () => {
    // Capture values delivered by the debounced callback.
    const calls = [];
    const debounced = debounce((value) => calls.push(value), 0);

    // Cancel the scheduled callback before its delay completes.
    debounced("pending");
    debounced.cancel();
    await waitForMicrotask();

    // Verify cancellation prevents callback execution.
    expect(calls).to.deep.equal([]);
  });

  it("adds and replaces query-string parameters", () => {
    // Start from a URL containing the parameter that will be replaced.
    setLocationPath("/jobs?sort=date&offset=10");

    // Replace the sort value while recording history state.
    addParamToQueryString("sort", "salary", { source: "unit" });

    // Verify unrelated parameters and history state are preserved.
    expect(window.location.pathname).to.equal("/jobs");
    expect(window.location.search).to.equal("?offset=10&sort=salary");
    expect(history.state).to.deep.equal({ source: "unit" });
  });

  it("preserves existing ids and generates deterministic missing ids", () => {
    // Build controls with existing and missing identifiers.
    const existingElement = document.createElement("button");
    existingElement.id = "existing-id";
    const firstElement = document.createElement("button");
    const secondElement = document.createElement("button");
    const options = {
      prefix: "generated",
      counterKey: "__unitElementIdCounter",
    };

    // Verify existing ids remain stable and generated ids increment.
    expect(ensureElementId({ element: existingElement, ...options })).to.equal("existing-id");
    expect(ensureElementId({ element: firstElement, ...options })).to.equal("generated-1");
    expect(ensureElementId({ element: secondElement, ...options })).to.equal("generated-2");
  });

  it("returns an empty id when no element is provided", () => {
    // Verify absent controls do not consume a generated identifier.
    expect(
      ensureElementId({
        element: null,
        prefix: "generated",
        counterKey: "__unitElementIdCounter",
      }),
    ).to.equal("");
  });

  it("triggers HTMX actions on existing forms", () => {
    // Build the form targeted by the HTMX action.
    document.body.innerHTML = '<form id="filters"></form>';

    // Trigger actions for an existing and an absent form.
    triggerActionOnForm("filters", "submit");
    triggerActionOnForm("missing", "submit");

    // Verify only the existing form emits the requested action.
    expect(htmx.triggerCalls).to.have.length(1);
    expect(htmx.triggerCalls[0]).to.deep.equal([document.getElementById("filters"), "submit"]);
  });

  it("converts normalized labels back to display text", () => {
    // Verify every normalized separator becomes display whitespace.
    expect(unnormalize("remote-friendly-health-care")).to.equal("remote friendly health care");
  });

  it("prettifies large numbers while preserving small values", () => {
    // Verify suffixes and requested decimal precision across size boundaries.
    expect(prettifyNumber(999)).to.equal(999);
    expect(prettifyNumber(1200)).to.equal("1.2k");
    expect(prettifyNumber(1500000)).to.equal("1.5M");
    expect(prettifyNumber(1234, 2)).to.equal("1.23k");
  });

  it("keeps drawer, backdrop, aria, and body state synchronized", () => {
    // Build the closed drawer and backdrop fixture.
    document.body.innerHTML = `
      <aside id="drawer" class="-translate-x-full" data-open="false" aria-hidden="true"></aside>
      <div id="drawer-backdrop" class="hidden"></div>
    `;

    // Open the drawer twice to exercise the duplicate-state guard.
    setDrawerVisibility({ drawerId: "drawer", open: true });
    setDrawerVisibility({ drawerId: "drawer", open: true });

    // Verify visible, accessible, and scroll-lock state stays synchronized.
    const drawer = document.getElementById("drawer");
    expect(drawer.dataset.open).to.equal("true");
    expect(drawer.getAttribute("aria-hidden")).to.equal("false");
    expect(drawer.classList.contains("-translate-x-full")).to.equal(false);
    expect(document.getElementById("drawer-backdrop").classList.contains("hidden")).to.equal(false);
    expect(document.body.style.overflow).to.equal("hidden");
    expect(document.body.dataset.modalOpenCount).to.equal("1");

    // Close the drawer after the guarded open transition.
    setDrawerVisibility({ drawerId: "drawer", open: false });

    // Verify closing restores hidden state and body scrolling.
    expect(drawer.dataset.open).to.equal("false");
    expect(drawer.getAttribute("aria-hidden")).to.equal("true");
    expect(drawer.classList.contains("-translate-x-full")).to.equal(true);
    expect(document.getElementById("drawer-backdrop").classList.contains("hidden")).to.equal(true);
    expect(document.body.style.overflow).to.equal("");
    expect(document.body.dataset.modalOpenCount).to.equal("0");
  });
});
