import { expect } from "@open-wc/testing";

import { passwordsMatch, trimmedNonEmpty } from "/static/js/common/validators.js";

describe("validators", () => {
  describe("trimmedNonEmpty", () => {
    it("rejects an empty value", () => {
      // Verify empty input uses the required-value message.
      expect(trimmedNonEmpty("")).to.equal("Value cannot be empty");
    });

    it("rejects a whitespace-only value", () => {
      // Verify whitespace-only input uses the required-value message.
      expect(trimmedNonEmpty("   ")).to.equal("Value cannot be empty");
    });

    it("accepts a non-empty value", () => {
      // Verify meaningful input has no validation error.
      expect(trimmedNonEmpty("GitJobs")).to.equal(null);
    });
  });

  describe("passwordsMatch", () => {
    it("accepts matching passwords", () => {
      // Verify equal password values have no validation error.
      expect(passwordsMatch("secret", "secret")).to.equal(null);
    });

    it("rejects different passwords", () => {
      // Verify unequal password values use the mismatch message.
      expect(passwordsMatch("secret", "different")).to.equal("Passwords do not match");
    });

    it("accepts two empty passwords", () => {
      // Verify two absent password values remain valid.
      expect(passwordsMatch("", "")).to.equal(null);
    });
  });
});
