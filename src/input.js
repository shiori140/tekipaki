import { syncLang, t, translateError, translatePage } from "./i18n.js";

const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

const form = document.getElementById("form");
const text = document.getElementById("text");
const hint = document.getElementById("hint");
const chips = [...form.querySelectorAll(".chip")];

let priority = "now";
let sourceApp = null;

function select(p) {
  priority = p;
  form.dataset.priority = p;
  chips.forEach((c) => c.setAttribute("aria-pressed", String(c.dataset.priority === p)));
}

async function save() {
  try {
    await invoke("save_manual", { text: text.value, priority, sourceApp });
  } catch (e) {
    hint.textContent = translateError(e);
    hint.classList.add("error");
  }
}

listen("input:open", (e) => {
  select(e.payload.priority);
  sourceApp = e.payload.source_app;
  text.value = "";
  hint.textContent = t("input.hint");
  hint.classList.remove("error");
  text.focus();
});

chips.forEach((c) =>
  c.addEventListener("click", () => {
    select(c.dataset.priority);
    text.focus();
  }),
);

text.addEventListener("keydown", (e) => {
  // 日本語入力の変換確定の Enter で登録されないようにする。
  if (e.isComposing) return;
  if (e.key === "Enter" && !e.shiftKey) {
    e.preventDefault();
    save();
  } else if (e.key === "Escape") {
    invoke("hide_input");
  }
});

await syncLang(() => {
  translatePage();
  hint.classList.remove("error");
});
translatePage();
select("now");
