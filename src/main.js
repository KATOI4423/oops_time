const { invoke } = window.__TAURI__.core;

// メニューバーをクリックした時に、ページを切り替える処理を追加する
document.addEventListener("DOMContentLoaded", () =>{
  const menus = document.querySelectorAll("nav button");
  const main = document.querySelector("main");

  // mainを更新する関数
  async function loadPage(url) {
    try {
      const res = await fetch(url);
      const html = await res.text();
      main.innerHTML = html;
    } catch (err) {
      main.innerHTML = `<p style="color: red;">ページの読み込みに失敗しました: ${err.status}</p>`;
    }
  }

  // 初期ページ読み込み
  loadPage("setting.html");

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
      loadPage(page);
    });
  });
});
