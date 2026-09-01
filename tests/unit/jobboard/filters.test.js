import { expect } from "@open-wc/testing";

import {
  cleanInputField,
  closeFiltersDrawer,
  openFiltersDrawer,
  resetForm,
  searchOnEnter,
  triggerActionOnForm,
  updateResults,
} from "/static/js/jobboard/filters.js";
import { resetDom } from "/tests/unit/test-utils/dom.js";
import { mockHtmx } from "/tests/unit/test-utils/globals.js";

describe("jobboard filters", () => {
  let htmx;

  beforeEach(() => {
    resetDom();
    htmx = mockHtmx();
  });

  afterEach(() => {
    htmx.restore();
    resetDom();
  });

  it("synchronizes the mobile drawer and backdrop state", () => {
    // Build the closed filters drawer and backdrop fixture.
    document.body.innerHTML = `
      <aside
        id="drawer-filters"
        class="-translate-x-full"
        data-open="false"
        aria-hidden="true"
      ></aside>
      <div id="drawer-backdrop" class="hidden"></div>
    `;

    // Open the filters drawer through its public helper.
    openFiltersDrawer();

    // Verify visible, accessible, and scroll-lock state opens together.
    const drawer = document.getElementById("drawer-filters");
    expect(drawer.dataset.open).to.equal("true");
    expect(drawer.getAttribute("aria-hidden")).to.equal("false");
    expect(document.getElementById("drawer-backdrop").classList.contains("hidden")).to.equal(false);
    expect(document.body.style.overflow).to.equal("hidden");

    // Close the filters drawer through its public helper.
    closeFiltersDrawer();

    // Verify closing restores hidden state and body scrolling.
    expect(drawer.dataset.open).to.equal("false");
    expect(drawer.getAttribute("aria-hidden")).to.equal("true");
    expect(document.getElementById("drawer-backdrop").classList.contains("hidden")).to.equal(true);
    expect(document.body.style.overflow).to.equal("");
  });

  it("does not submit empty searches", () => {
    // Build a search fixture containing only whitespace.
    document.body.innerHTML = `
      <form id="filters"></form>
      <input id="searchbar" value="   " />
    `;

    // Request a form action from the search path.
    triggerActionOnForm("filters", "submit", true);

    // Verify empty search values do not submit.
    expect(htmx.triggerCalls).to.deep.equal([]);
  });

  it("trims and submits non-empty searches", () => {
    // Build a search fixture containing padded meaningful text.
    document.body.innerHTML = `
      <form id="filters"></form>
      <input id="searchbar" value="  platform engineer  " />
    `;

    // Request a form action from the search path.
    triggerActionOnForm("filters", "submit", true);

    // Verify search text is normalized before form submission.
    expect(document.getElementById("searchbar").value).to.equal("platform engineer");
    expect(htmx.triggerCalls).to.deep.equal([[document.getElementById("filters"), "submit"]]);
  });

  it("submits and blurs search input on Enter", () => {
    // Build and focus the keyboard-driven search fixture.
    document.body.innerHTML = `
      <form id="filters"></form>
      <input id="searchbar" value="engineer" />
    `;
    const input = document.getElementById("searchbar");
    input.addEventListener("keydown", (event) => searchOnEnter(event, "filters"));
    input.focus();

    // Dispatch the Enter key through the public keyboard handler.
    input.dispatchEvent(new KeyboardEvent("keydown", { bubbles: true, key: "Enter" }));

    // Verify Enter submits the form and releases input focus.
    expect(htmx.triggerCalls).to.deep.equal([[document.getElementById("filters"), "submit"]]);
    expect(document.activeElement).to.not.equal(input);
  });

  it("ignores search keys other than Enter", () => {
    // Build a search fixture wired to the keyboard handler.
    document.body.innerHTML = `
      <form id="filters"></form>
      <input id="searchbar" value="engineer" />
    `;
    const input = document.getElementById("searchbar");
    input.addEventListener("keydown", (event) => searchOnEnter(event, "filters"));

    // Dispatch a navigation key that should not submit.
    input.dispatchEvent(new KeyboardEvent("keydown", { bubbles: true, key: "ArrowDown" }));

    // Verify non-Enter keys leave the form untouched.
    expect(htmx.triggerCalls).to.deep.equal([]);
  });

  it("clears an input and optionally submits its form", () => {
    // Build the input and associated form fixture.
    document.body.innerHTML = `
      <form id="filters"></form>
      <input id="query" value="engineer" />
    `;

    // Clear an existing input and ignore an absent input.
    cleanInputField("query", "filters");
    cleanInputField("missing", "filters");

    // Verify the real input clears and submits only once.
    expect(document.getElementById("query").value).to.equal("");
    expect(htmx.triggerCalls).to.deep.equal([[document.getElementById("filters"), "submit"]]);
  });

  it("updates results when the results container exists", () => {
    // Build the server-results container fixture.
    document.body.innerHTML = '<div id="results">Old results</div>';

    // Replace the current results with the supplied markup.
    updateResults("<strong>New results</strong>");

    // Verify existing containers update and absent containers are safe.
    expect(document.getElementById("results").innerHTML).to.equal("<strong>New results</strong>");

    document.getElementById("results").remove();
    expect(() => updateResults("Ignored")).not.to.throw();
  });

  it("resets native and custom form controls before submitting", async () => {
    // Build native fields and custom filters associated with the form.
    document.body.innerHTML = `
      <form id="filters"></form>
      <select name="date_range" form="filters"><option value="last30-days">30 days</option></select>
      <select id="sort" form="filters"><option value="salary" selected>Salary</option></select>
      <input id="check" form="filters" type="checkbox" checked />
      <input id="radio" form="filters" type="radio" checked />
      <input id="text" form="filters" type="text" value="engineer" />
      <input id="hidden" form="filters" type="hidden" value="hidden-value" />
      <input id="searchbar" value="main query" />
      <searchable-filter></searchable-filter>
      <search-projects></search-projects>
      <input-range></input-range>
      <search-location></search-location>
    `;

    // Record cleanup calls exposed by each custom filter contract.
    const resetCalls = [];
    document.querySelector("searchable-filter").cleanSelected = async () => resetCalls.push("filter");
    document.querySelector("search-projects").cleanSelected = async () => resetCalls.push("projects");
    document.querySelector("input-range").cleanRange = async () => resetCalls.push("range");
    document.querySelector("search-location").cleanLocation = async () => resetCalls.push("location");

    // Reset the complete filter form through its public helper.
    await resetForm("filters");

    // Verify native values, custom controls, and submission stay synchronized.
    expect(document.querySelector('[name="date_range"]').value).to.equal("last30-days");
    expect(document.getElementById("sort").value).to.equal("");
    expect(document.getElementById("check").checked).to.equal(false);
    expect(document.getElementById("radio").checked).to.equal(false);
    expect(document.getElementById("text").value).to.equal("");
    expect(document.getElementById("hidden").value).to.equal("");
    expect(document.getElementById("searchbar").value).to.equal("");
    expect(resetCalls).to.deep.equal(["filter", "projects", "range", "location"]);
    expect(htmx.triggerCalls).to.deep.equal([[document.getElementById("filters"), "submit"]]);
  });

  it("ignores reset requests for missing forms", async () => {
    // Request cleanup for a form that is not present.
    await resetForm("missing");

    // Verify absent forms do not emit an HTMX action.
    expect(htmx.triggerCalls).to.deep.equal([]);
  });
});
