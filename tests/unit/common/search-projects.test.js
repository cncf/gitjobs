import { expect } from "@open-wc/testing";

import "/static/js/common/search-projects.js";
import { waitForMicrotask, waitForTimeout } from "/tests/unit/test-utils/async.js";
import { mockHtmx } from "/tests/unit/test-utils/globals.js";
import { mountLitComponent, useMountedElementsCleanup } from "/tests/unit/test-utils/lit.js";

const projectLogoDataUrl = "data:image/svg+xml,%3Csvg xmlns=%22http://www.w3.org/2000/svg%22/%3E";

describe("search-projects", () => {
  let fetchCalls;
  let fetchImplementation;
  let htmx;
  let originalFetch;

  useMountedElementsCleanup("search-projects");

  beforeEach(() => {
    originalFetch = globalThis.fetch;
    fetchCalls = [];
    fetchImplementation = async () => ({
      json: async () => [],
      ok: true,
    });
    globalThis.fetch = (...args) => {
      fetchCalls.push(args);
      return fetchImplementation(...args);
    };
    htmx = mockHtmx();
  });

  afterEach(() => {
    globalThis.fetch = originalFetch;
    htmx.restore();
  });

  const createDeferred = () => {
    let resolve;
    const promise = new Promise((promiseResolve) => {
      resolve = promiseResolve;
    });
    return { promise, resolve };
  };

  const createProject = (name, foundation = "cncf") => ({
    foundation,
    logo_url: projectLogoDataUrl,
    maturity: "graduated",
    name,
  });

  const renderSearch = async (properties = {}) => {
    document.body.innerHTML = `<form id="filters">
      <select name="foundation">
        <option value=""></option>
        <option value="cncf" selected>CNCF</option>
      </select>
    </form>`;
    return mountLitComponent("search-projects", {
      form: "filters",
      foundations: [{ name: "cncf" }, { name: "linux-foundation" }],
      ...properties,
    });
  };

  const selectFoundation = async (element, foundation = "cncf") => {
    const select = element.querySelector("select");
    select.value = foundation;
    select.dispatchEvent(new Event("change", { bubbles: true }));
    await element.updateComplete;
  };

  const enterSearch = async (element, value) => {
    const input = element.querySelector('input[type="text"]');
    input.value = value;
    input.dispatchEvent(new Event("input", { bubbles: true }));
    await element.updateComplete;
    return input;
  };

  it("renders in light DOM and enables search after choosing a foundation", async () => {
    // Render the project search without an active foundation.
    const element = await renderSearch();

    // Verify the light-DOM search starts disabled.
    expect(element.shadowRoot).to.equal(null);
    expect(element.querySelector('input[type="text"]').disabled).to.equal(true);

    // Select a foundation through the rendered control.
    await selectFoundation(element);

    // Verify foundation state enables project search.
    expect(element.selectedFoundation).to.equal("cncf");
    expect(element.querySelector('input[type="text"]').disabled).to.equal(false);
  });

  it("does not request projects for fewer than three characters", async () => {
    // Render an enabled project-search fixture.
    const element = await renderSearch();
    await selectFoundation(element);

    // Enter a short query and wait past the debounce delay.
    await enterSearch(element, "ab");
    await waitForTimeout(320);

    // Verify short queries leave network and dropdown state untouched.
    expect(fetchCalls).to.have.length(0);
    expect(element.visibleOptions).to.equal(null);
    expect(element.visibleDropdown).to.equal(false);
  });

  it("requests and renders projects for the current foundation", async () => {
    // Render an enabled search and prepare a successful response.
    const element = await renderSearch();
    const project = createProject("kubernetes");
    fetchImplementation = async () => ({
      json: async () => [project],
      ok: true,
    });
    await selectFoundation(element);

    // Enter a searchable value and complete the debounced request.
    await enterSearch(element, "kube & tools");
    await waitForTimeout(320);
    await waitForMicrotask();
    await element.updateComplete;

    // Verify query encoding, visible results, and image text contracts.
    expect(fetchCalls[0][0]).to.equal("/projects/search?project=kube%20%26%20tools&foundation=cncf");
    expect(element.visibleOptions).to.deep.equal([project]);
    expect(element.visibleDropdown).to.equal(true);
    expect(element.querySelector("li button").textContent).to.contain("kubernetes");
    expect(element.querySelector("li img").alt).to.equal("kubernetes logo");
  });

  it("shows an empty result after a failed request", async () => {
    // Render an enabled search with a failing response.
    const element = await renderSearch();
    fetchImplementation = async () => ({ ok: false, status: 503 });
    await selectFoundation(element);

    // Enter a searchable value and complete the failed request.
    await enterSearch(element, "missing");
    await waitForTimeout(320);
    await waitForMicrotask();
    await element.updateComplete;

    // Verify failures resolve to the visible empty state.
    expect(element.visibleOptions).to.deep.equal([]);
    expect(element.visibleDropdown).to.equal(true);
    expect(element.textContent).to.contain("No projects found");
  });

  it("ignores a stale response when a newer search has completed", async () => {
    // Prepare independently controlled responses for overlapping searches.
    const element = await renderSearch();
    const firstRequest = createDeferred();
    const secondRequest = createDeferred();
    const secondProject = createProject("second-project");
    fetchImplementation = () => (fetchCalls.length === 1 ? firstRequest.promise : secondRequest.promise);
    await selectFoundation(element);

    // Start two searches without resolving the first request.
    await enterSearch(element, "first");
    await waitForTimeout(320);
    await enterSearch(element, "second");
    await waitForTimeout(320);

    // Resolve the newer request before the original request.
    secondRequest.resolve({
      json: async () => [secondProject],
      ok: true,
    });
    await waitForMicrotask();
    await element.updateComplete;

    // Resolve the stale request after the current result is rendered.
    firstRequest.resolve({
      json: async () => [{ foundation: "cncf", name: "first-project" }],
      ok: true,
    });
    await waitForMicrotask();
    await element.updateComplete;

    // Verify the older response cannot replace the current result.
    expect(element.visibleOptions).to.deep.equal([secondProject]);
  });

  it("invalidates an outstanding response when the foundation changes", async () => {
    // Prepare a controlled request for the initial foundation.
    const element = await renderSearch();
    const request = createDeferred();
    fetchImplementation = () => request.promise;
    await selectFoundation(element);

    // Start a search and switch foundations before its response arrives.
    await enterSearch(element, "project");
    await waitForTimeout(320);
    await selectFoundation(element, "linux-foundation");

    // Resolve the response belonging to the previous foundation.
    request.resolve({
      json: async () => [{ foundation: "cncf", name: "old-project" }],
      ok: true,
    });
    await waitForMicrotask();
    await element.updateComplete;

    // Verify foundation changes preserve the reset search state.
    expect(element.selectedFoundation).to.equal("linux-foundation");
    expect(element.visibleOptions).to.equal(null);
    expect(element.visibleDropdown).to.equal(false);
  });

  it("cancels pending work and removes outside-click handling on disconnect", async () => {
    // Start a debounced search on a connected component.
    const element = await renderSearch();
    await selectFoundation(element);
    await enterSearch(element, "project");

    // Disconnect the component before the request delay completes.
    element.remove();
    element.enteredValue = "detached";
    document.body.dispatchEvent(new MouseEvent("mousedown", { bubbles: true }));
    await waitForTimeout(320);

    // Verify pending fetch and global click behavior are cleaned up.
    expect(fetchCalls).to.have.length(0);
    expect(element.enteredValue).to.equal("detached");
  });

  it("selects the active project with the keyboard and submits the form", async () => {
    // Render a visible result for keyboard selection.
    const element = await renderSearch();
    await selectFoundation(element);
    element.visibleOptions = [createProject("kubernetes")];
    element.visibleDropdown = true;
    await element.updateComplete;
    const input = element.querySelector('input[type="text"]');

    // Highlight and select the result through the keyboard contract.
    input.dispatchEvent(new KeyboardEvent("keydown", { bubbles: true, key: "ArrowDown" }));
    input.dispatchEvent(new KeyboardEvent("keydown", { bubbles: true, cancelable: true, key: "Enter" }));
    await waitForMicrotask();
    await element.updateComplete;

    // Read the actual form payload after selection.
    const formData = new FormData(document.getElementById("filters"));

    // Verify selection, submitted values, cleanup, and HTMX action.
    expect(element.selected).to.have.length(1);
    expect(formData.get("projects[0][name]")).to.equal("kubernetes");
    expect(formData.get("projects[0][foundation]")).to.equal("cncf");
    expect(document.querySelector('[name="foundation"]').value).to.equal("");
    expect(htmx.triggerCalls).to.deep.equal([[document.getElementById("filters"), "submit"]]);
  });

  it("clears selected projects and returns to the disabled state", async () => {
    // Render a project search with an existing selection.
    const element = await renderSearch({
      selected: [{ foundation: "cncf", name: "kubernetes" }],
      selectedFoundation: "cncf",
    });

    // Clear selection through the public cleanup method.
    await element.cleanSelected();

    // Read the associated form after the component update.
    const formData = new FormData(document.getElementById("filters"));

    // Verify selection, input, and submitted values reset together.
    expect(element.selected).to.deep.equal([]);
    expect(element.selectedFoundation).to.equal(null);
    expect(element.querySelector('input[type="text"]').disabled).to.equal(true);
    expect(formData.has("projects[0][name]")).to.equal(false);
  });
});
