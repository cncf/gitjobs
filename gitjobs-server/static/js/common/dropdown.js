// Highlight the item in the list when the user uses the arrow keys
export const highlightItem = (id, direction) => {
  const list = document.querySelector(`#${id} ul`);
  if (list) {
    const numItems = list.querySelectorAll("li").length;
    const highlightedItem = list.querySelector("li.active");
    if (highlightedItem) {
      const currentActiveIndex = parseInt(highlightedItem.dataset.index);
      let newIndex = direction === "up" ? currentActiveIndex - 1 : currentActiveIndex + 1;
      if (newIndex > numItems) {
        newIndex = 1;
      }
      if (newIndex <= 0) {
        newIndex = numItems;
      }
      highlightedItem.classList.remove("active");
      const newActiveItem = list.querySelector(`li:nth-child(${newIndex})`);
      if (newActiveItem) {
        newActiveItem.classList.add("active");
        newActiveItem.scrollIntoView({ behavior: "instant", block: "nearest", inline: "start" });
      }
    } else {
      list.querySelector(`li:${direction === "down" ? "first-child" : "last-child"}`).classList.add("active");
    }
  }
};

export const addCard = (id, name, label, logo_url, elId, onRemove, extra = "") => {
  const card = `
  <div id="card-${id}" class="relative border rounded-lg p-4 pe-10 bg-white mt-4 min-w-64">
    <button id="remove-${id}" data-id="${id}" type="button" class="rounded-full bg-gray-100 hover:bg-gray-200 absolute top-1 end-1">
      <div class="svg-icon size-5 bg-gray-400 hover:bg-gray-700 icon-close"></div>
    </button>
    <div class="flex items-center space-x-3">
      <img class="size-10"
          height="40"
          width="40"
          src="${logo_url}"
          alt="${name} logo">
      <div class="flex flex-col justify-start min-w-0">
        <div class="truncate text-start text-gray-700 font-medium">${name}</div>
        <div class="inline-flex">
          <div class="truncate text-nowrap uppercase max-w-[100%] text-xs/6 font-medium text-gray-400">
            ${label}
          </div>
        </div>
      </div>
    </div>
    ${extra}
  </div>
  `;

  const el = document.getElementById(elId);
  el.insertAdjacentHTML("beforeend", card);

  const removeButton = document.getElementById(`remove-${id}`);
  removeButton.addEventListener("click", () => {
    el.removeChild(removeButton.parentElement);
    onRemove(id);
  });
};
