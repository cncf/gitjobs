/**
 * Trims string values and removes empty/"0" entries before HTMX serialization.
 * @param {FormData|URLSearchParams} source - Source parameters
 * @returns {Array<[string, string|File]>} Filtered key/value entries
 */
const removeEmptyValues = (source) => {
  const filteredEntries = [];

  for (const [key, rawValue] of source.entries()) {
    const value = typeof rawValue === "string" ? rawValue.trim() : String(rawValue);
    if (value === "" || value === "0") {
      continue;
    }

    filteredEntries.push([key, typeof rawValue === "string" ? value : rawValue]);
  }

  return filteredEntries;
};

/**
 * Registers the HTMX extension that removes empty values from request parameters.
 */
export const initializeHtmxNoEmptyValues = () => {
  if (window.__gitjobsNoEmptyValsExtensionBound) {
    return;
  }

  htmx.defineExtension("no-empty-vals", {
    onEvent: (name, event) => {
      if (name !== "htmx:configRequest") {
        return true;
      }

      const request = event.detail;
      if (request.verb !== "get" || !request.useUrlParams) {
        return true;
      }

      const filteredParameters = new FormData();
      for (const [key, value] of removeEmptyValues(request.formData)) {
        filteredParameters.append(key, value);
      }

      request.formData = filteredParameters;
      request.parameters = filteredParameters;

      return true;
    },
    encodeParameters: (xhr, parameters, elt) => {
      const filteredEntries = removeEmptyValues(parameters);

      for (const key of [...parameters.keys()]) {
        parameters.delete(key);
      }

      for (const [key, value] of filteredEntries) {
        parameters.append(key, value);
      }

      return null;
    },
  });

  window.__gitjobsNoEmptyValsExtensionBound = true;
};

initializeHtmxNoEmptyValues();
