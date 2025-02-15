import { addCard } from "/static/js/common/dropdown.js";

export const addMemberCard = (id, name, level, logo_url, elId) => {
  // Remove previous selected member
  document.getElementById("selected-member").innerHTML = "";

  addCard(id, name, level, logo_url, elId, removeSelectedMember);
};

export const removeSelectedMember = () => {
  const memberInput = document.getElementById("member");
  const memberIdInput = document.getElementById("member_id");
  const memberNameInput = document.getElementById("member_name");
  const memberLevelInput = document.getElementById("member_level");
  const memberLogoUrlInput = document.getElementById("member_logo_url");
  const contentData = "search-member";

  // Clear member inputs
  memberInput.value = "";
  memberIdInput.value = "";
  memberNameInput.value = "";
  memberLevelInput.value = "";
  memberLogoUrlInput.value = "";
  // Remove members dropdown
  document.getElementById(contentData).innerHTML = "";
  // Remove selected member card
  document.getElementById("selected-member").innerHTML = "";
};
