import { t } from "./i18n.js";

export const COLUMNS = ["now", "today", "later", "done"];

/** 完了列は振り返り用なので、新しいものから一定数だけ出す。 */
export const DONE_LIMIT = 50;

const MINUTE = 60_000;
const HOUR = 60 * MINUTE;
const DAY = 24 * HOUR;

export function groupTasks(tasks) {
  const groups = Object.fromEntries(COLUMNS.map((c) => [c, []]));
  for (const t of tasks) {
    if (t.status === "done") groups.done.push(t);
    else groups[t.priority].push(t);
  }
  for (const c of ["now", "today", "later"]) {
    groups[c].sort((a, b) => b.createdAt.localeCompare(a.createdAt));
  }
  groups.done.sort((a, b) => (b.completedAt ?? "").localeCompare(a.completedAt ?? ""));
  groups.done = groups.done.slice(0, DONE_LIMIT);
  return groups;
}

export function elapsed(iso, now = Date.now()) {
  const ms = now - new Date(iso).getTime();
  if (ms < MINUTE) return t("elapsed.justNow");
  if (ms < HOUR) return t("elapsed.minutes", { n: Math.floor(ms / MINUTE) });
  if (ms < DAY) return t("elapsed.hours", { n: Math.floor(ms / HOUR) });
  return t("elapsed.days", { n: Math.floor(ms / DAY) });
}

/** 放置の度合い。0: 1時間未満 / 1: 6時間未満 / 2: 1日未満 / 3: 1日以上 */
export function ageLevel(iso, now = Date.now()) {
  const ms = now - new Date(iso).getTime();
  if (ms < HOUR) return 0;
  if (ms < 6 * HOUR) return 1;
  if (ms < DAY) return 2;
  return 3;
}

/** 列の件数に収まるよう、選択位置を補正する。 */
export function clampSelection(sel, groups) {
  const col = Math.min(Math.max(sel.col, 0), COLUMNS.length - 1);
  const len = groups[COLUMNS[col]].length;
  const row = len === 0 ? 0 : Math.min(Math.max(sel.row, 0), len - 1);
  return { col, row };
}

const MOVES = {
  ArrowUp: [0, -1],
  k: [0, -1],
  ArrowDown: [0, 1],
  j: [0, 1],
  ArrowLeft: [-1, 0],
  h: [-1, 0],
  ArrowRight: [1, 0],
  l: [1, 0],
};

/** 移動キーなら新しい選択位置を、そうでなければ null を返す。 */
export function moveSelection(sel, key, groups) {
  const move = MOVES[key];
  if (!move) return null;
  const [dc, dr] = move;
  return clampSelection({ col: sel.col + dc, row: sel.row + dr }, groups);
}

export function locate(id, groups) {
  for (const [col, name] of COLUMNS.entries()) {
    const row = groups[name].findIndex((t) => t.id === id);
    if (row !== -1) return { col, row };
  }
  return null;
}
