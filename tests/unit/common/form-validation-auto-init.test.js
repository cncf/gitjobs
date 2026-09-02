import { expect } from "@open-wc/testing";

import { resetDom } from "/tests/unit/test-utils/dom.js";
import { stubValidityUi } from "/tests/unit/test-utils/forms.js";

describe("form validation auto-initialization", () => {
  afterEach(() => {
    resetDom();
  });

  it("wires forms that exist before the module loads", async () => {
    // Build the form fixture before importing the self-initializing module.
    document.body.innerHTML = `
      <form id="profile-form">
        <input id="display-name" name="display_name" required value="   " />
      </form>
    `;
    const form = document.getElementById("profile-form");
    const input = document.getElementById("display-name");
    stubValidityUi(input);

    // Load the module and complete deferred DOM-ready initialization if needed.
    await import("/static/js/common/form-validation.js");
    if (form.dataset.trimmedReady !== "true") {
      document.dispatchEvent(new Event("DOMContentLoaded"));
    }

    // Submit the invalid pre-existing form through its public event contract.
    const submitEvent = new Event("submit", { bubbles: true, cancelable: true });
    form.dispatchEvent(submitEvent);

    // Verify auto-initialization wires validation and its public error message.
    expect(form.dataset.trimmedReady).to.equal("true");
    expect(submitEvent.defaultPrevented).to.equal(true);
    expect(input.validationMessage).to.equal("Value cannot be empty");
  });
});
