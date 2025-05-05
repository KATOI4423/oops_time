// メニューバーをクリックした時に、ページを切り替える処理を追加する
document.addEventListener("DOMContentLoaded", () =>{
  const menus = document.querySelectorAll("nav button");
  const frame = document.getElementById("page-frame");

  // ボタンクリック時の処理
  menus.forEach(item => {
    item.addEventListener("click", () => {
      // 既に選択中の場合、何もしない
      if (item.classList.contains("current"))
        return;

      // クラスの切り替え
      menus.forEach(itm => itm.classList.remove("current"));
      item.classList.add("current");

      // ページ読み込み
      const page = item.getAttribute("data-page");
      frame.src = page;
    });
  });
});
