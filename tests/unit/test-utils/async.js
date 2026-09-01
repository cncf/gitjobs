/** Waits for pending microtasks and zero-delay timers to flush. */
export const waitForMicrotask = () => new Promise((resolve) => setTimeout(resolve, 0));

/** Waits for one or more animation frames used by DOM-driven updates. */
export const waitForAnimationFrames = async (count = 2) => {
  for (let index = 0; index < count; index += 1) {
    await new Promise((resolve) => requestAnimationFrame(resolve));
  }
};

/** Waits for a specific timer delay. */
export const waitForTimeout = (delay) => new Promise((resolve) => setTimeout(resolve, delay));
