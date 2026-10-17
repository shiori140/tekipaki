import { syncLang, t, translatePage } from "./i18n.js";
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

const PRIORITY_KEYS = { 1: "now", 2: "today", 3: "later" };
const UNDO_LIMIT = 50;

const isMac = navigator.userAgent.includes("Mac");
const mod = isMac ? "⌘" : "Ctrl+";

const columnsEl = document.getElementById("columns");
const toast = document.getElementById("toast");
const toastText = document.getElementById("toast-text");

/** タスク一覧以外の、言語を切り替えたときに描き直す部分。 */
function renderChrome() {
  translatePage();
  document.getElementById("shortcut").textContent = t("board.howTo", {
    shortcut: isMac ? "⌘⇧Space" : "Ctrl+Shift+Space",
  });
  document.getElementById("keys").replaceChildren(
    ...[
      ["↑↓←→", "keys.move"],
      ["1 2 3", "keys.priority"],
      ["Space", "keys.toggle"],
      [`${mod}Z`, "keys.undo"],
    ].map(([k, key]) => {
      const span = el("span");
      span.append(el("kbd", null, k), ` ${t(key)}`);
      return span;
    }),
  );
}

let groups = groupTasks([]);
let sel = { col: 0, row: 0 };
const undoStack = [];
let toastTimer;

function columnLabel(name) {
  return name === "done" ? t("column.done") : t(`priority.${name}`);
}

function el(tag, className, text) {
  const node = document.createElement(tag);
  if (className) node.className = className;
  if (text) node.textContent = text;
  return node;
}

function selectedTask() {
  return groups[COLUMNS[sel.col]][sel.row] ?? null;
}

function renderTask(task, col, row) {
  const li = el("li", "task");
  li.dataset.priority = task.priority;
  li.classList.toggle("done", task.status === "done");
  li.classList.toggle("selected", col === sel.col && row === sel.row);

  const check = el("button", "check", "✓");
  check.type = "button";
  check.tabIndex = -1;
  check.title = t(task.status === "done" ? "board.markOpen" : "board.markDone");
  check.addEventListener("click", (e) => {
    e.stopPropagation();
    sel = { col, row };
    toggleDone();
  });

  const body = el("div", "body");
  body.append(el("div", "title", task.title));
  if (task.memo) body.append(el("div", "memo", task.memo));
  const meta = el("div", "meta");
  if (task.sourceApp) meta.append(`${task.sourceApp} · `);
  if (task.status === "done") {
    meta.append(t("board.completedAt", { elapsed: elapsed(task.completedAt) }));
  } else {
    const age = el("span", "age", elapsed(task.createdAt));
    age.dataset.level = String(ageLevel(task.createdAt));
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
      const h2 = el("h2", null, columnLabel(name));
      h2.append(el("span", "count", String(items.length)));
      const list = el("ul");
      if (items.length === 0) list.append(el("li", "empty", t("board.empty")));
      items.forEach((task, row) => list.append(renderTask(task, col, row)));
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
    showToast(t("toast.changeFailed", { error: e }), false);
  }
}

function toggleDone() {
  const task = selectedTask();
  if (!task) return;
  const done = task.status === "done";
  change(task, { status: done ? "open" : "done" }, t(done ? "toast.reopened" : "toast.done"));
}

function setPriority(priority) {
  const task = selectedTask();
  if (!task) return;
  if (task.priority === priority && task.status === "open") return;
  change(task, { priority, status: "open" }, t("toast.moved", { priority: t(`priority.${priority}`) }));
}

async function undo() {
  const before = undoStack.pop();
  if (!before) {
    showToast(t("toast.nothingToUndo"), false);
    return;
  }
  try {
    await invoke("restore_task", { task: before });
    await reload();
    sel = locate(before.id, groups) ?? sel;
    render();
    showToast(t("toast.undone"), false);
  } catch (e) {
    undoStack.push(before);
    showToast(t("toast.undoFailed", { error: e }), false);
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
await syncLang(() => {
  renderChrome();
  render();
});
renderChrome();
reload();
