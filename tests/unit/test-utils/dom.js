/** Resets the DOM and shared body state between unit test cases. */
export const resetDom = () => {
  document.body.innerHTML = "";
  document.body.removeAttribute("style");
  delete document.body.dataset.modalOpenCount;
  delete document.body.dataset.modalOverflow;
  delete document.body.dataset.modalPaddingRight;
  document.documentElement.removeAttribute("style");
};

/** Updates the current same-origin path without triggering navigation. */
export const setLocationPath = (path) => {
  history.replaceState({}, "", path);
};
