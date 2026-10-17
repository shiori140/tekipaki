import { PRIORITIES, PRIORITY_ORDER } from "./priority.js";

const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

const columns = document.getElementById("columns");
const isMac = navigator.userAgent.includes("Mac");
document.getElementById("shortcut").textContent =
  `文字を選んで ${isMac ? "⌘⇧Space" : "Ctrl+Shift+Space"} を押すと登録（1回: 今すぐ / 2回: 今日中 / 3回: あとで）`;

function elapsed(iso) {
  const minutes = Math.floor((Date.now() - new Date(iso).getTime()) / 60000);
  if (minutes < 1) return "たった今";
  if (minutes < 60) return `${minutes}分前`;
  const hours = Math.floor(minutes / 60);
  if (hours < 24) return `${hours}時間前`;
  return `${Math.floor(hours / 24)}日前`;
}

function el(tag, className, text) {
  const node = document.createElement(tag);
  if (className) node.className = className;
  if (text) node.textContent = text;
  return node;
}

async function render() {
  const tasks = (await invoke("list_tasks"))
    .filter((t) => t.status === "open")
    .sort((a, b) => b.createdAt.localeCompare(a.createdAt));

  columns.replaceChildren(
    ...PRIORITY_ORDER.map((priority) => {
      const items = tasks.filter((t) => t.priority === priority);
      const section = el("section");
      section.dataset.priority = priority;
      const h2 = el("h2", null, PRIORITIES[priority].label);
      h2.append(el("span", "count", String(items.length)));
      const list = el("ul");
      if (items.length === 0) list.append(el("li", "empty", "なし"));
      for (const t of items) {
        const li = el("li");
        li.append(el("div", "title", t.title));
        if (t.memo) li.append(el("div", "memo", t.memo));
        const meta = [t.sourceApp, elapsed(t.createdAt)].filter(Boolean).join(" ・ ");
        li.append(el("div", "meta", meta));
        list.append(li);
      }
      section.append(h2, list);
      return section;
    }),
  );
}

listen("tasks:changed", render);
// 経過時間の表示を更新する。
setInterval(render, 60_000);
render();
