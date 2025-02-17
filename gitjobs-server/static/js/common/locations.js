// Highlight the item in the list when the user uses the arrow keys
export const highlightItem = (direction) => {
  const locationList = document.querySelector("#search-location ul");
  if (locationList) {
    const numItems = locationList.querySelectorAll("li").length;
    const highlightedItem = locationList.querySelector("li.active");
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
      const newActiveItem = locationList.querySelector(`li:nth-child(${newIndex})`);
      if (newActiveItem) {
        newActiveItem.classList.add("active");
        newActiveItem.scrollIntoView({
          behavior: "instant",
          block: "nearest",
          inline: "start",
        });
      }
    } else {
      locationList
        .querySelector(`li:${direction === "down" ? "first-child" : "last-child"}`)
        .classList.add("active");
    }
  }
};
