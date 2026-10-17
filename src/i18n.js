// 言語を足すときは、ここに同じキーで辞書を追加し、SUPPORTED にも加える。
const MESSAGES = {
  ja: {
    "priority.now": "今すぐ",
    "priority.today": "今日中",
    "priority.later": "あとで",
    "column.done": "完了",

    "elapsed.justNow": "たった今",
    "elapsed.minutes": "{n}分前",
    "elapsed.hours": "{n}時間前",
    "elapsed.days": "{n}日前",

    "board.howTo": "文字を選んで {shortcut} で登録（1回: 今すぐ / 2回: 今日中 / 3回: あとで）",
    "board.empty": "なし",
    "board.completedAt": "完了 {elapsed}",
    "board.markDone": "完了にする",
    "board.markOpen": "未完了に戻す",
    "keys.move": "移動",
    "keys.priority": "優先度を変更",
    "keys.toggle": "完了 / 未完了",
    "keys.undo": "元に戻す",

    "toast.done": "完了にしました",
    "toast.reopened": "未完了に戻しました",
    "toast.moved": "「{priority}」に移しました",
    "toast.undone": "元に戻しました",
    "toast.nothingToUndo": "元に戻せる操作はありません",
    "toast.undo": "元に戻す",
    "toast.changeFailed": "変更できませんでした: {error}",
    "toast.undoFailed": "元に戻せませんでした: {error}",

    "input.placeholder": "タスクを入力（Enterで登録・Escで閉じる）",
    "input.hint": "Shift+Enterで改行",

    "overlay.failed": "登録できませんでした",

    "error.empty": "内容が空です",
    "error.noPermission": "アクセシビリティの許可が必要です",
    "error.saveFailed": "保存に失敗しました: {detail}",
  },
  en: {
    "priority.now": "Now",
    "priority.today": "Today",
    "priority.later": "Later",
    "column.done": "Done",

    "elapsed.justNow": "just now",
    "elapsed.minutes": "{n}m ago",
    "elapsed.hours": "{n}h ago",
    "elapsed.days": "{n}d ago",

    "board.howTo": "Select text and press {shortcut} (once: Now / twice: Today / 3 times: Later)",
    "board.empty": "Nothing here",
    "board.completedAt": "Done {elapsed}",
    "board.markDone": "Mark as done",
    "board.markOpen": "Mark as not done",
    "keys.move": "Move",
    "keys.priority": "Change priority",
    "keys.toggle": "Done / not done",
    "keys.undo": "Undo",

    "toast.done": "Marked as done",
    "toast.reopened": "Marked as not done",
    "toast.moved": "Moved to \"{priority}\"",
    "toast.undone": "Undone",
    "toast.nothingToUndo": "Nothing to undo",
    "toast.undo": "Undo",
    "toast.changeFailed": "Couldn't update: {error}",
    "toast.undoFailed": "Couldn't undo: {error}",

    "input.placeholder": "Type a task (Enter to add, Esc to close)",
    "input.hint": "Shift+Enter for a new line",

    "overlay.failed": "Couldn't add the task",

    "error.empty": "The task is empty",
    "error.noPermission": "Accessibility permission is required",
    "error.saveFailed": "Couldn't save: {detail}",
  },
};

const SUPPORTED = Object.keys(MESSAGES);
const FALLBACK = "en";

/** OS の言語設定（例: "ja-JP"）から、対応している言語を選ぶ。 */
export function detectLang(languages) {
  for (const tag of languages ?? []) {
    const base = String(tag).toLowerCase().split("-")[0];
    if (SUPPORTED.includes(base)) return base;
  }
  return FALLBACK;
}

let lang = detectLang(globalThis.navigator?.languages ?? [globalThis.navigator?.language]);

export function getLang() {
  return lang;
}

export function setLang(next) {
  lang = SUPPORTED.includes(next) ? next : FALLBACK;
  if (globalThis.document) document.documentElement.lang = lang;
}

export function t(key, params = {}) {
  const template = MESSAGES[lang][key] ?? MESSAGES[FALLBACK][key] ?? key;
  return template.replace(/\{(\w+)\}/g, (_, name) => String(params[name] ?? `{${name}}`));
}

/** Rust から届くエラーの種類を翻訳する。辞書にない種類は、そのまま表示する。 */
export function translateError(code, params = {}) {
  const key = `error.${code}`;
  return key in MESSAGES[lang] || key in MESSAGES[FALLBACK] ? t(key, params) : String(code);
}

/** HTML に書いた data-i18n / data-i18n-placeholder の文言を差し替える。 */
export function translatePage(root = document) {
  document.documentElement.lang = lang;
  for (const node of root.querySelectorAll("[data-i18n]")) {
    node.textContent = t(node.dataset.i18n);
  }
  for (const node of root.querySelectorAll("[data-i18n-placeholder]")) {
    node.placeholder = t(node.dataset.i18nPlaceholder);
  }
}
