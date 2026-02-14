/**
 * Frontend validators returning error messages or null.
 * Browser handles URL, email, and length validation via native attributes.
 * @module validators
 */

/**
 * Validates that a string is non-empty after trimming whitespace.
 * @param {string} value - The value to validate
 * @returns {string|null} Error message if invalid, null if valid
 */
export const trimmedNonEmpty = (value) => {
  if (!value || value.trim() === "") {
    return "Value cannot be empty";
  }

  return null;
};

/**
 * Validates that two passwords match.
 * @param {string} password - The password value
 * @param {string} confirmation - The confirmation value
 * @returns {string|null} Error message if invalid, null if valid
 */
export const passwordsMatch = (password, confirmation) => {
  if (password !== confirmation) {
    return "Passwords do not match";
  }

  return null;
};
