import { PRIORITIES } from "./priority.js";
import {
  ageLevel,
  clampSelection,
  COLUMNS,
  elapsed,
  groupTasks,
  locate,
  moveSelection,
} from "./board.js";

const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

const COLUMN_LABELS = { ...Object.fromEntries(Object.entries(PRIORITIES).map(([k, v]) => [k, v.label])), done: "完了" };
const PRIORITY_KEYS = { 1: "now", 2: "today", 3: "later" };
const UNDO_LIMIT = 50;

const isMac = navigator.userAgent.includes("Mac");
const mod = isMac ? "⌘" : "Ctrl+";

const columnsEl = document.getElementById("columns");
const toast = document.getElementById("toast");
const toastText = document.getElementById("toast-text");

document.getElementById("shortcut").textContent =
  `文字を選んで ${isMac ? "⌘⇧Space" : "Ctrl+Shift+Space"} で登録（1回: 今すぐ / 2回: 今日中 / 3回: あとで）`;
document.getElementById("keys").innerHTML = [
  ["↑↓←→", "移動"],
  ["1 2 3", "優先度を変更"],
  ["Space", "完了 / 未完了"],
  [`${mod}Z`, "元に戻す"],
]
  .map(([k, label]) => `<span><kbd>${k}</kbd> ${label}</span>`)
  .join("");

let groups = groupTasks([]);
let sel = { col: 0, row: 0 };
const undoStack = [];
let toastTimer;

function el(tag, className, text) {
  const node = document.createElement(tag);
  if (className) node.className = className;
  if (text) node.textContent = text;
  return node;
}

function selectedTask() {
  return groups[COLUMNS[sel.col]][sel.row] ?? null;
}

function renderTask(t, col, row) {
  const li = el("li", "task");
  li.dataset.priority = t.priority;
  li.classList.toggle("done", t.status === "done");
  li.classList.toggle("selected", col === sel.col && row === sel.row);

  const check = el("button", "check", "✓");
  check.type = "button";
  check.tabIndex = -1;
  check.title = t.status === "done" ? "未完了に戻す" : "完了にする";
  check.addEventListener("click", (e) => {
    e.stopPropagation();
    sel = { col, row };
    toggleDone();
  });

  const body = el("div", "body");
  body.append(el("div", "title", t.title));
  if (t.memo) body.append(el("div", "memo", t.memo));
  const meta = el("div", "meta");
  if (t.sourceApp) meta.append(`${t.sourceApp} ・ `);
  if (t.status === "done") {
    meta.append(`完了 ${elapsed(t.completedAt)}`);
  } else {
    const age = el("span", "age", elapsed(t.createdAt));
    age.dataset.level = String(ageLevel(t.createdAt));
    meta.append(age);
  }
  body.append(meta);

  li.append(check, body);
  li.addEventListener("click", () => {
    sel = { col, row };
    render();
  });
  return li;
}

function render() {
  sel = clampSelection(sel, groups);
  columnsEl.replaceChildren(
    ...COLUMNS.map((name, col) => {
      const items = groups[name];
      const section = el("section");
      section.dataset.column = name;
      const h2 = el("h2", null, COLUMN_LABELS[name]);
      h2.append(el("span", "count", String(items.length)));
      const list = el("ul");
      if (items.length === 0) list.append(el("li", "empty", "なし"));
      items.forEach((t, row) => list.append(renderTask(t, col, row)));
      section.append(h2, list);
      return section;
    }),
  );
  columnsEl.querySelector(".selected")?.scrollIntoView({ block: "nearest" });
}

async function reload() {
  groups = groupTasks(await invoke("list_tasks"));
  render();
}

function showToast(message, canUndo) {
  toastText.textContent = message;
  document.getElementById("toast-undo").hidden = !canUndo;
  toast.classList.add("visible");
  clearTimeout(toastTimer);
  toastTimer = setTimeout(() => toast.classList.remove("visible"), 3000);
}

/** 確認ダイアログは出さず、変更前の状態を積んでおいて「元に戻す」で取り消せるようにする。 */
async function change(task, patch, message) {
  try {
    const before = await invoke("update_task", { id: task.id, ...patch });
    if (!before) return;
    undoStack.push(before);
    if (undoStack.length > UNDO_LIMIT) undoStack.shift();
    await reload();
    // どれが動いたか見失わないよう、移動先の列でも同じタスクを選んだままにする。
    sel = locate(task.id, groups) ?? sel;
    render();
    showToast(message, true);
  } catch (e) {
    showToast(`変更できませんでした: ${e}`, false);
  }
}

function toggleDone() {
  const t = selectedTask();
  if (!t) return;
  const done = t.status === "done";
  change(t, { status: done ? "open" : "done" }, done ? "未完了に戻しました" : "完了にしました");
}

function setPriority(priority) {
  const t = selectedTask();
  if (!t) return;
  if (t.priority === priority && t.status === "open") return;
  change(t, { priority, status: "open" }, `「${PRIORITIES[priority].label}」に移しました`);
}

async function undo() {
  const before = undoStack.pop();
  if (!before) {
    showToast("元に戻せる操作はありません", false);
    return;
  }
  try {
    await invoke("restore_task", { task: before });
    await reload();
    sel = locate(before.id, groups) ?? sel;
    render();
    showToast("元に戻しました", false);
  } catch (e) {
    undoStack.push(before);
    showToast(`元に戻せませんでした: ${e}`, false);
  }
}

document.getElementById("toast-undo").addEventListener("click", undo);

document.addEventListener("keydown", (e) => {
  if (e.isComposing) return;
  if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "z" && !e.shiftKey) {
    e.preventDefault();
    undo();
    return;
  }
  if (e.metaKey || e.ctrlKey || e.altKey) return;

  const moved = moveSelection(sel, e.key, groups);
  if (moved) {
    e.preventDefault();
    sel = moved;
    render();
  } else if (PRIORITY_KEYS[e.key]) {
    e.preventDefault();
    setPriority(PRIORITY_KEYS[e.key]);
  } else if (e.key === " " || e.key === "Enter" || e.key === "x") {
    e.preventDefault();
    toggleDone();
  } else if (e.key === "u") {
    e.preventDefault();
    undo();
  }
});

listen("tasks:changed", reload);
// 経過時間の表示を更新する。
setInterval(render, 60_000);
reload();
