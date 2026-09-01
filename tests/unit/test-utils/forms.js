/** Records native validity UI requests without opening browser validation UI. */
export const stubValidityUi = (field) => {
  const calls = [];
  field.reportValidity = () => {
    calls.push(true);
    return true;
  };
  return { calls };
};
