const { invoke } = window.__TAURI__.core;

let greetInputEl;
let greetMsgEl;

async function greet() {
  // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
  greetMsgEl.textContent = await invoke("greet", { name: greetInputEl.value });
}

window.addEventListener("DOMContentLoaded", () => {
  greetInputEl = document.querySelector("#greet-input");
  greetMsgEl = document.querySelector("#greet-msg");
  document.querySelector("#greet-form").addEventListener("submit", (e) => {
    e.preventDefault();
    greet();
  });
});

document.getElementById("notify").addEventListener("click", async () =>
{
	const title = document.getElementById("title").value || "Notification";
	const body  = document.getElementById("body").value  || "Test Notification";

	try {
		await invoke("send_notify", {title, body});
		console.log("Sent notification.");
	} catch (e) {
		console.error("Failed to send notification:", e);
	}
});
