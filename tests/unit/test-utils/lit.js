import { resetDom } from "/tests/unit/test-utils/dom.js";

/** Mounts a Lit element with properties and waits for its first render. */
export const mountLitComponent = async (tagName, properties = {}) => {
  const element = document.createElement(tagName);
  Object.assign(element, properties);
  document.body.append(element);
  await element.updateComplete;
  return element;
};

/** Registers mounted-element cleanup and DOM reset hooks for a suite. */
export const useMountedElementsCleanup = (...selectors) => {
  afterEach(() => {
    selectors.forEach((selector) => {
      document.querySelectorAll(selector).forEach((element) => element.remove());
    });
    resetDom();
  });
};
