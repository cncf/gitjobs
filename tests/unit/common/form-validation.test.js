import { expect } from "@open-wc/testing";

import { resetDom } from "/tests/unit/test-utils/dom.js";
import { stubValidityUi } from "/tests/unit/test-utils/forms.js";

describe("form validation", () => {
  let originalHtmx;
  let wireLoadedContent;

  before(async () => {
    originalHtmx = window.htmx;
    window.htmx = {
      onLoad: (callback) => {
        wireLoadedContent = callback;
      },
    };
    await import("/static/js/common/form-validation.js");
  });

  beforeEach(() => {
    resetDom();
  });

  after(() => {
    resetDom();
    window.htmx = originalHtmx;
  });

  const dispatchConfigRequest = (target, requestElement = target, bubbles = false) => {
    const event = new CustomEvent("htmx:configRequest", {
      bubbles,
      cancelable: true,
      detail: { elt: requestElement },
    });
    target.dispatchEvent(event);
    return event;
  };

  const dispatchSubmit = (form) => {
    const event = new Event("submit", { bubbles: true, cancelable: true });
    form.dispatchEvent(event);
    return event;
  };

  const renderForm = (fields) => {
    document.body.innerHTML = `<form id="test-form">${fields}</form>`;
    const form = document.getElementById("test-form");
    wireLoadedContent(form);
    return form;
  };

  it("trims optional non-password values before submission", () => {
    // Render an optional field containing surrounding whitespace.
    const form = renderForm('<input id="name" name="name" value="  GitJobs  " />');
    const input = document.getElementById("name");

    // Submit the form through its wired event listener.
    const submitEvent = dispatchSubmit(form);

    // Verify optional input is normalized without blocking submission.
    expect(submitEvent.defaultPrevented).to.equal(false);
    expect(input.value).to.equal("GitJobs");
  });

  it("trims valid required non-password values", () => {
    // Render a required field containing meaningful padded text.
    const form = renderForm('<input id="name" name="name" required value="  GitJobs  " />');
    const input = document.getElementById("name");
    stubValidityUi(input);

    // Submit the form through its wired event listener.
    const submitEvent = dispatchSubmit(form);

    // Verify valid required input is normalized and accepted.
    expect(submitEvent.defaultPrevented).to.equal(false);
    expect(input.value).to.equal("GitJobs");
  });

  it("rejects required whitespace-only values with the expected message", () => {
    // Render a required field containing only whitespace.
    const form = renderForm('<input id="name" name="name" required value="   " />');
    const input = document.getElementById("name");
    const validityUi = stubValidityUi(input);

    // Submit the invalid form through its wired event listener.
    const submitEvent = dispatchSubmit(form);

    // Verify submission stops and exposes the stable validation message.
    expect(submitEvent.defaultPrevented).to.equal(true);
    expect(input.validationMessage).to.equal("Value cannot be empty");
    expect(validityUi.calls).to.have.length(1);
  });

  it("preserves intentional whitespace in password values", () => {
    // Render matching password fields with intentional surrounding spaces.
    const form = renderForm(`
      <input id="password" type="password" required data-password value=" secret " />
      <input
        id="confirmation"
        type="password"
        required
        data-password-confirmation
        value=" secret "
      />
    `);
    const passwordInput = document.getElementById("password");
    const confirmationInput = document.getElementById("confirmation");
    stubValidityUi(passwordInput);
    stubValidityUi(confirmationInput);

    // Submit the matching password fixture.
    const submitEvent = dispatchSubmit(form);

    // Verify password values remain untrimmed and submission continues.
    expect(submitEvent.defaultPrevented).to.equal(false);
    expect(passwordInput.value).to.equal(" secret ");
    expect(confirmationInput.value).to.equal(" secret ");
  });

  it("rejects mismatching password confirmation", () => {
    // Render password fields with different values.
    const form = renderForm(`
      <input id="password" type="password" data-password value="secret" />
      <input id="confirmation" type="password" data-password-confirmation value="different" />
    `);
    const confirmationInput = document.getElementById("confirmation");
    const validityUi = stubValidityUi(confirmationInput);

    // Submit the mismatching password fixture.
    const submitEvent = dispatchSubmit(form);

    // Verify submission stops and reports the password contract message.
    expect(submitEvent.defaultPrevented).to.equal(true);
    expect(confirmationInput.validationMessage).to.equal("Passwords do not match");
    expect(validityUi.calls).to.have.length(1);
  });

  it("keeps password confirmation validity synchronized while typing", () => {
    // Render password fields with an initial mismatch.
    renderForm(`
      <input id="password" type="password" data-password value="secret" />
      <input id="confirmation" type="password" data-password-confirmation value="different" />
    `);
    const passwordInput = document.getElementById("password");
    const confirmationInput = document.getElementById("confirmation");

    // Revalidate the mismatching confirmation value.
    confirmationInput.dispatchEvent(new Event("input", { bubbles: true }));
    expect(confirmationInput.validationMessage).to.equal("Passwords do not match");

    // Match the confirmation and verify its validation error clears.
    confirmationInput.value = "secret";
    confirmationInput.dispatchEvent(new Event("input", { bubbles: true }));
    expect(confirmationInput.validationMessage).to.equal("");

    // Clear the primary password and verify an incomplete pair stays neutral.
    passwordInput.value = "";
    passwordInput.dispatchEvent(new Event("input", { bubbles: true }));
    expect(confirmationInput.validationMessage).to.equal("");
  });

  it("prevents invalid HTMX form requests", () => {
    // Render a form with an invalid required field.
    const form = renderForm('<input id="name" name="name" required value="   " />');
    const input = document.getElementById("name");
    stubValidityUi(input);

    // Dispatch the HTMX request event from the form.
    const requestEvent = dispatchConfigRequest(form);

    // Verify the request stops with the public validation message.
    expect(requestEvent.defaultPrevented).to.equal(true);
    expect(input.validationMessage).to.equal("Value cannot be empty");
  });

  it("skips HTMX validation for the cancel button", () => {
    // Render an invalid form with the reserved cancel control.
    const form = renderForm(`
      <input id="name" name="name" required value="   " />
      <button id="cancel-button" type="button">Cancel</button>
    `);
    const input = document.getElementById("name");
    stubValidityUi(input);

    // Dispatch the request on behalf of the cancel control.
    const requestEvent = dispatchConfigRequest(form, document.getElementById("cancel-button"));

    // Verify cancellation bypasses form validation.
    expect(requestEvent.defaultPrevented).to.equal(false);
    expect(input.validationMessage).to.equal("");
  });

  it("skips HTMX validation for declarative skip controls", () => {
    // Render an invalid form with a declarative validation bypass.
    const form = renderForm(`
      <input id="name" name="name" required value="   " />
      <button id="skip" type="button" data-skip-validation="true">Skip</button>
    `);
    const input = document.getElementById("name");
    stubValidityUi(input);

    // Dispatch the request on behalf of the skip control.
    const requestEvent = dispatchConfigRequest(form, document.getElementById("skip"));

    // Verify the declarative bypass leaves the request untouched.
    expect(requestEvent.defaultPrevented).to.equal(false);
    expect(input.validationMessage).to.equal("");
  });

  it("prevents requests that include an invalid form", () => {
    // Build an HTMX control that includes a separate invalid form.
    document.body.innerHTML = `
      <form id="included-form">
        <input id="included-name" name="name" required value="   " />
      </form>
      <button id="request" type="button" hx-include="#included-form">Save</button>
    `;
    const input = document.getElementById("included-name");
    stubValidityUi(input);

    // Bubble the request event through the module-level HTMX listener.
    const requestEvent = dispatchConfigRequest(document.getElementById("request"), undefined, true);

    // Verify invalid included form data stops the request.
    expect(requestEvent.defaultPrevented).to.equal(true);
    expect(input.validationMessage).to.equal("Value cannot be empty");
  });

  it("allows requests with valid included forms and trims their values", () => {
    // Build an HTMX control that includes a separate valid form.
    document.body.innerHTML = `
      <form id="included-form">
        <input id="included-name" name="name" required value="  GitJobs  " />
      </form>
      <button id="request" type="button" hx-include="#included-form">Save</button>
    `;
    const input = document.getElementById("included-name");
    stubValidityUi(input);

    // Bubble the request event through the module-level HTMX listener.
    const requestEvent = dispatchConfigRequest(document.getElementById("request"), undefined, true);

    // Verify included values are normalized without blocking the request.
    expect(requestEvent.defaultPrevented).to.equal(false);
    expect(input.value).to.equal("GitJobs");
  });

  it("ignores disabled invalid fields", () => {
    // Render a disabled required field containing an invalid value.
    const form = renderForm('<input id="name" name="name" required disabled value="   " />');
    const input = document.getElementById("name");
    stubValidityUi(input);

    // Submit the form through its wired event listener.
    const submitEvent = dispatchSubmit(form);

    // Verify disabled fields do not participate in validation.
    expect(submitEvent.defaultPrevented).to.equal(false);
    expect(input.validationMessage).to.equal("");
  });

  it("does not wire the same form twice", () => {
    // Render a form and invoke the swapped-content initializer again.
    const form = renderForm('<input id="name" name="name" required value="   " />');
    const input = document.getElementById("name");
    const validityUi = stubValidityUi(input);
    wireLoadedContent(form);

    // Submit the invalid form after duplicate initialization.
    const submitEvent = dispatchSubmit(form);

    // Verify the guard keeps a single validation listener active.
    expect(form.dataset.trimmedReady).to.equal("true");
    expect(submitEvent.defaultPrevented).to.equal(true);
    expect(validityUi.calls).to.have.length(1);
  });
});
