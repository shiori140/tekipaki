import { syncLang, t, translateError } from "./i18n.js";
import { PRIORITIES } from "./priority.js";

const { listen } = window.__TAURI__.event;

const card = document.getElementById("card");
const label = document.getElementById("label");
const preview = document.getElementById("preview");
const steps = [...card.querySelectorAll(".steps span")];

function render({ priority, preview: text }) {
  const p = PRIORITIES[priority];
  const changed = card.dataset.priority !== priority;
  card.dataset.priority = priority;
  label.textContent = t(`priority.${priority}`);
  steps.forEach((dot, i) => dot.classList.toggle("on", i < p.step));
  preview.textContent = text ?? "";
  if (changed) {
    // 同じクラスを付け直してもアニメーションは再生されないため、一度外して描画させる。
    card.classList.remove("bump");
    void card.offsetWidth;
    card.classList.add("bump");
  }
}

listen("overlay:show", () => {
  delete card.dataset.state;
  card.dataset.priority = "";
  preview.textContent = "";
});

listen("overlay:update", (e) => {
  if (!card.dataset.state) render(e.payload);
});

listen("overlay:commit", () => {
  card.dataset.state = "commit";
});

listen("overlay:cancel", () => {
  card.dataset.state = "cancel";
});

// Rust からはエラーの種類（code）だけを受け取り、文言はここで翻訳する。
listen("overlay:error", (e) => {
  const { code, detail } = e.payload;
  card.dataset.state = "error";
  label.textContent = t("overlay.failed");
  preview.textContent = translateError(code, { detail });
  steps.forEach((dot) => dot.classList.remove("on"));
});

await syncLang();
