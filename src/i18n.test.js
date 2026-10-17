import test from "node:test";
import assert from "node:assert/strict";

import { elapsed } from "./board.js";
import { detectLang, getLang, setLang, t, translateError } from "./i18n.js";

test("picks a supported language from the OS language list", () => {
  assert.equal(detectLang(["ja-JP", "en-US"]), "ja");
  assert.equal(detectLang(["en-GB"]), "en");
  assert.equal(detectLang(["fr-FR", "ja"]), "ja");
  assert.equal(detectLang(["fr-FR"]), "en");
  assert.equal(detectLang([]), "en");
});

test("every language has the same keys", async () => {
  const source = await import("node:fs").then((fs) =>
    fs.readFileSync(new URL("./i18n.js", import.meta.url), "utf8"),
  );
  const keysOf = (lang) => {
    const block = source.split(`  ${lang}: {`)[1].split("\n  },")[0];
    return [...block.matchAll(/^\s+"([\w.]+)":/gm)].map((m) => m[1]).sort();
  };
  assert.deepEqual(keysOf("en"), keysOf("ja"));
});

test("translates with parameters in each language", () => {
  const now = Date.parse("2026-10-18T12:00:00Z");
  setLang("en");
  assert.equal(getLang(), "en");
  assert.equal(t("toast.moved", { priority: t("priority.today") }), 'Moved to "Today"');
  assert.equal(elapsed("2026-10-18T09:00:00Z", now), "3h ago");
  setLang("ja");
  assert.equal(t("toast.moved", { priority: t("priority.today") }), "「今日中」に移しました");
  assert.equal(elapsed("2026-10-18T09:00:00Z", now), "3時間前");
});

test("translates known error codes and passes unknown ones through", () => {
  setLang("en");
  assert.equal(translateError("noPermission"), "Accessibility permission is required");
  assert.equal(translateError("saveFailed", { detail: "disk full" }), "Couldn't save: disk full");
  assert.equal(translateError("something odd"), "something odd");
});
