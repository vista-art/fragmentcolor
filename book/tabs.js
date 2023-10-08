document.addEventListener("DOMContentLoaded", function () {
  const tabs = document.querySelectorAll(".tab");
  tabs.forEach((tab) => {
    tab.addEventListener("click", function () {
      const target = tab.getAttribute("data-target");
      document.querySelectorAll(".code-block").forEach((block) => {
        block.classList.remove("active");
      });
      document.querySelectorAll(`.${target}`).forEach((block) => {
        block.classList.add("active");
      });
      tabs.forEach((innerTab) => {
        innerTab.classList.remove("active");
        if (innerTab.getAttribute("data-target") === target) {
          innerTab.classList.add("active");
        }
      });
    });
  });
});
