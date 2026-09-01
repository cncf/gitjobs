import { expect } from "@open-wc/testing";

import { initializePendingChangesAlert } from "/static/js/dashboard/jobseeker/pending-changes-alert.js";
import { waitForAnimationFrames, waitForMicrotask } from "/tests/unit/test-utils/async.js";
import { resetDom } from "/tests/unit/test-utils/dom.js";
import { mockHtmx, mockSwal } from "/tests/unit/test-utils/globals.js";

describe("pending-changes-alert", () => {
  let htmx;
  let swal;

  beforeEach(() => {
    htmx = mockHtmx();
    swal = mockSwal();
  });

  afterEach(() => {
    htmx.restore();
    swal.restore();
    resetDom();
  });

  const renderFixture = ({ includeAlert = true, includeCancel = true } = {}) => {
    document.body.innerHTML = `${includeAlert ? '<div id="pending-alert" class="hidden"></div>' : ""}
      ${includeCancel ? '<button id="cancel" type="button">Cancel</button>' : ""}
      <form id="profile-form">
        <input name="display_name" value="Ada">
        <input name="location" value="London">
      </form>
      <form id="preferences-form">
        <input name="role" value="Engineer">
      </form>`;

    return initializePendingChangesAlert({
      alertId: "pending-alert",
      cancelButtonId: includeCancel ? "cancel" : "",
      confirmMessage: "Discard your changes?",
      confirmText: "Discard",
      formIds: ["profile-form", "preferences-form"],
    });
  };

  const initializeFixture = async (options) => {
    const api = renderFixture(options);
    await waitForAnimationFrames(2);
    return api;
  };

  const updateInput = async (selector, value) => {
    const input = document.querySelector(selector);
    input.value = value;
    input.dispatchEvent(new Event("input", { bubbles: true }));
    await waitForAnimationFrames(2);
    return input;
  };

  it("shows pending state for edits and clears it after reverting", async () => {
    // Initialize tracking from the server-rendered values.
    const api = await initializeFixture();
    const alert = document.getElementById("pending-alert");

    // Verify the initial snapshot is clean and hidden.
    expect(api.hasPendingChanges()).to.equal(false);
    expect(alert.classList.contains("hidden")).to.equal(true);

    // Change a tracked value and verify pending state becomes visible.
    await updateInput('[name="display_name"]', "Grace");

    expect(api.hasPendingChanges()).to.equal(true);
    expect(alert.classList.contains("hidden")).to.equal(false);

    // Restore the original value and verify clean state returns.
    await updateInput('[name="display_name"]', "Ada");

    expect(api.hasPendingChanges()).to.equal(false);
    expect(alert.classList.contains("hidden")).to.equal(true);
  });

  it("tracks changes across every configured form", async () => {
    // Initialize tracking for both fixture forms.
    const api = await initializeFixture();

    // Change a value in the secondary tracked form.
    await updateInput('#preferences-form [name="role"]', "Maintainer");

    // Verify secondary-form changes mark the page dirty.
    expect(api.hasPendingChanges()).to.equal(true);
  });

  it("detects dynamically added named fields", async () => {
    // Initialize tracking before creating an additional field.
    const api = await initializeFixture();
    const input = document.createElement("input");
    input.name = "timezone";
    input.value = "Europe/Madrid";

    // Add submitted data and allow mutation tracking to refresh.
    document.getElementById("profile-form").append(input);
    await waitForAnimationFrames(2);

    // Verify new submitted data marks the form state dirty.
    expect(api.hasPendingChanges()).to.equal(true);
  });

  it("keeps the snapshot clean when fields are only reordered", async () => {
    // Initialize tracking from the original field order.
    const api = await initializeFixture();
    const form = document.getElementById("profile-form");

    // Move a field without changing its submitted name or value.
    form.append(form.querySelector('[name="display_name"]'));
    await waitForAnimationFrames(2);

    // Verify deterministic snapshots ignore DOM ordering changes.
    expect(api.hasPendingChanges()).to.equal(false);
  });

  it("marks the current values as clean and tracks later edits", async () => {
    // Initialize tracking and create a pending change.
    const api = await initializeFixture();
    await updateInput('[name="location"]', "Madrid");

    // Promote the edited values to the new clean snapshot.
    api.markCurrentAsClean();

    // Verify pending state and the alert clear immediately.
    expect(api.hasPendingChanges()).to.equal(false);
    expect(document.getElementById("pending-alert").classList.contains("hidden")).to.equal(true);

    // Apply another edit after the new clean snapshot.
    await updateInput('[name="location"]', "Paris");

    // Verify later changes are still detected.
    expect(api.hasPendingChanges()).to.equal(true);
  });

  it("allows cancel without confirmation while the forms are clean", async () => {
    // Initialize clean tracking and prepare a cancel click.
    await initializeFixture();
    const event = new MouseEvent("click", { bubbles: true, cancelable: true });

    // Dispatch cancel before any tracked values change.
    document.getElementById("cancel").dispatchEvent(event);

    // Verify clean cancellation continues without confirmation.
    expect(event.defaultPrevented).to.equal(false);
    expect(swal.calls).to.have.length(0);
    expect(htmx.triggerCalls).to.have.length(0);
  });

  it("confirms cancel and emits the confirmed event while dirty", async () => {
    // Initialize tracking and make the form dirty.
    await initializeFixture();
    await updateInput('[name="location"]', "Madrid");
    const cancelButton = document.getElementById("cancel");
    const event = new MouseEvent("click", { bubbles: true, cancelable: true });

    // Dispatch cancel and allow confirmation to settle.
    cancelButton.dispatchEvent(event);
    await waitForMicrotask();

    // Verify dirty cancellation uses the configured confirmation.
    expect(event.defaultPrevented).to.equal(true);
    expect(swal.calls).to.have.length(1);
    expect(swal.calls[0].text).to.equal("Discard your changes?");
    expect(swal.calls[0].confirmButtonText).to.equal("Discard");
    expect(htmx.triggerCalls).to.deep.equal([[cancelButton, "confirmed"]]);
  });

  it("does not duplicate cancel handling when initialized twice", async () => {
    // Initialize the same alert twice with identical controls.
    const firstApi = await initializeFixture();
    const secondApi = initializePendingChangesAlert({
      alertId: "pending-alert",
      cancelButtonId: "cancel",
      confirmMessage: "Discard your changes?",
      confirmText: "Discard",
      formIds: ["profile-form", "preferences-form"],
    });

    // Make the tracked form dirty after duplicate initialization.
    await updateInput('[name="location"]', "Madrid");

    // Dispatch cancel and allow confirmation handling to settle.
    document
      .getElementById("cancel")
      .dispatchEvent(new MouseEvent("click", { bubbles: true, cancelable: true }));
    await waitForMicrotask();

    // Verify initialization reuses the API and one cancel handler.
    expect(secondApi).to.equal(firstApi);
    expect(swal.calls).to.have.length(1);
  });

  it("returns a safe inactive API when the alert is absent", async () => {
    // Initialize tracking without an alert or cancel control.
    const api = await initializeFixture({ includeAlert: false, includeCancel: false });

    // Exercise edits and refresh methods without the owner element.
    await updateInput('[name="display_name"]', "Grace");
    api.refresh();
    api.markCurrentAsClean();

    // Verify the inactive API remains clean and safe to call.
    expect(api.hasPendingChanges()).to.equal(false);
  });
});
