const { invoke } = window.__TAURI__.core;

document.getElementById("test-notify-button").addEventListener("click", async () => {
  const DEFAULT_TITLE = "OopsTime Notification Test";
  const DEFAULT_BODY  = "Notification has been sent successfully!";
  const title = document.getElementById("test-notify-title").value || DEFAULT_TITLE;
  const body  = document.getElementById("test-notify-body").value  || DEFAULT_BODY;

  try {
    await invoke("send_notify", {title, body});
  } catch (err) {
    console.error("Failed to send notifycation: ", err);
  }
});
