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

export const addCard = (id, name, label, logo_url, elId, onRemove, extra = "", mini = false) => {
  const card = `
  <div id="card-${id}" class="relative border border-stone-200 rounded-lg p-${mini ? "2" : "4"} pe-10 bg-white min-w-64">
    <button id="remove-${id}" data-id="${id}" type="button" class="rounded-full cursor-pointer bg-stone-100 hover:bg-stone-200 absolute ${
      mini ? "top-2 end-2" : "top-1 end-1"
    }">
      <div class="svg-icon size-5 bg-stone-400 hover:bg-stone-700 icon-close"></div>
    </button>
    <div class="flex items-center space-x-3">
      <div class="size-${mini ? "5" : "10"} shrink-0 flex items-center justify-center">
        <img class="size-${mini ? "5" : "10"} object-contain"
            height="auto"
            width="auto"
            src="${logo_url}"
            alt="${name} logo">
      </div>
      <div class="flex flex-col justify-start min-w-0">
        <div class="truncate text-start text-stone-700 font-medium ${mini ? "text-sm" : ""}">${name}</div>
        ${
          !mini
            ? `<div class="inline-flex">
          <div class="truncate text-nowrap uppercase max-w-[100%] text-xs/6 font-medium text-stone-500/75">
            ${label}
          </div>
        </div>`
            : ""
        }
      </div>
    </div>
    ${extra}
  </div>
  `;

  const el = document.getElementById(elId);
  if (el) {
    el.insertAdjacentHTML("beforeend", card);
  }

  const removeButton = document.getElementById(`remove-${id}`);
  removeButton.addEventListener("click", () => {
    el.removeChild(removeButton.parentElement);
    onRemove(id);
  });
};
