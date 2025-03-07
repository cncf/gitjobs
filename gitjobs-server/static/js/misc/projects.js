import { addCard } from "/static/js/common/dropdown.js";

export const addProjectCard = (id, name, maturity, logo_url, elId, mini = false) => {
  // Get the number of cards in the container
  // This will be used to set the index of the project in the form
  const cardsNumber = document.querySelectorAll(`#${elId} div[id^='card-']`).length;

  // Check if the card is already added
  const addedCard = document.getElementById(`card-${id}`);
  if (addedCard) {
    return;
  }

  let inputs = "";

  if (mini) {
    inputs = `
    <input type="hidden"
      name="projects[]"
      value="${name}">`;
  } else {
    inputs = `
    <input type="hidden"
      data-index="${cardsNumber}"
      name="projects[${cardsNumber}][project_id]"
      value="${id}">
    <input type="hidden" name="projects[${cardsNumber}][name]" value="${name}">
    <input type="hidden"
      name="projects[${cardsNumber}][maturity]"
      value="${maturity}">
    <input type="hidden"
      name="projects[${cardsNumber}][logo_url]"
      value="${logo_url}">`;
  }
  addCard(id, name, `CNCF ${maturity}`, logo_url, elId, removeSelectedProject, inputs, mini);
};

export const removeSelectedProject = (id) => {
  const cards = document.querySelectorAll(`#selected-projects div[id^='card-']`);
  cards.forEach((card, index) => {
    const inputs = card.querySelectorAll("input");
    const currentIndex = inputs[0].dataset.index;
    // If the current index is not the same as the index of the card
    // Update the index of the card and its inputs
    if (currentIndex !== index) {
      inputs[0].setAttribute("data-index", index);
      inputs.forEach((input) => {
        const name = input.getAttribute("name");
        input.setAttribute("name", name.replace(`[${currentIndex}]`, `[${index}]`));
      });
    }
  });
};
