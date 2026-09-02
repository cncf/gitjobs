/** Mocks HTMX helpers and records trigger calls. */
export const mockHtmx = () => {
  const originalHtmx = globalThis.htmx;
  const triggerCalls = [];

  globalThis.htmx = {
    trigger: (...args) => {
      triggerCalls.push(args);
    },
  };

  return {
    triggerCalls,
    restore() {
      globalThis.htmx = originalHtmx;
    },
  };
};

/** Mocks SweetAlert and records each dialog configuration. */
export const mockSwal = () => {
  const originalSwal = globalThis.Swal;
  const calls = [];
  let nextResult = { isConfirmed: true };

  globalThis.Swal = {
    fire: async (options) => {
      calls.push(options);
      return nextResult;
    },
  };

  return {
    calls,
    setNextResult(result) {
      nextResult = result;
    },
    restore() {
      globalThis.Swal = originalSwal;
    },
  };
};
