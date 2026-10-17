import test from "node:test";
import assert from "node:assert/strict";

import {
  ageLevel,
  clampSelection,
  DONE_LIMIT,
  elapsed,
  groupTasks,
  locate,
  moveSelection,
} from "./board.js";

const task = (id, priority, createdAt, extra = {}) => ({
  id,
  priority,
  status: "open",
  createdAt,
  completedAt: null,
  ...extra,
});

test("groups open tasks by priority, newest first, and done tasks separately", () => {
  const groups = groupTasks([
    task("a", "now", "2026-10-18T09:00:00+09:00"),
    task("b", "now", "2026-10-18T10:00:00+09:00"),
    task("c", "later", "2026-10-18T08:00:00+09:00"),
    task("d", "today", "2026-10-18T08:00:00+09:00", {
      status: "done",
      completedAt: "2026-10-18T11:00:00+09:00",
    }),
  ]);
  assert.deepEqual(groups.now.map((t) => t.id), ["b", "a"]);
  assert.deepEqual(groups.today, []);
  assert.deepEqual(groups.later.map((t) => t.id), ["c"]);
  assert.deepEqual(groups.done.map((t) => t.id), ["d"]);
});

test("limits the done column", () => {
  const many = Array.from({ length: DONE_LIMIT + 5 }, (_, i) =>
    task(String(i), "now", "2026-10-18T00:00:00Z", {
      status: "done",
      completedAt: new Date(Date.UTC(2026, 9, 18, 0, i)).toISOString(),
    }),
  );
  const done = groupTasks(many).done;
  assert.equal(done.length, DONE_LIMIT);
  assert.equal(done[0].id, String(DONE_LIMIT + 4));
});

test("formats elapsed time and age level", () => {
  const now = Date.parse("2026-10-18T12:00:00Z");
  assert.equal(elapsed("2026-10-18T11:59:30Z", now), "たった今");
  assert.equal(elapsed("2026-10-18T11:48:00Z", now), "12分前");
  assert.equal(elapsed("2026-10-18T09:00:00Z", now), "3時間前");
  assert.equal(elapsed("2026-10-16T12:00:00Z", now), "2日前");
  assert.equal(ageLevel("2026-10-18T11:30:00Z", now), 0);
  assert.equal(ageLevel("2026-10-18T10:00:00Z", now), 1);
  assert.equal(ageLevel("2026-10-18T00:00:00Z", now), 2);
  assert.equal(ageLevel("2026-10-17T00:00:00Z", now), 3);
});

test("moves and clamps the selection", () => {
  const groups = { now: [1, 2, 3], today: [1], later: [], done: [1, 2] };
  assert.deepEqual(moveSelection({ col: 0, row: 0 }, "ArrowDown", groups), { col: 0, row: 1 });
  assert.deepEqual(moveSelection({ col: 0, row: 2 }, "j", groups), { col: 0, row: 2 });
  assert.deepEqual(moveSelection({ col: 0, row: 2 }, "ArrowRight", groups), { col: 1, row: 0 });
  assert.deepEqual(moveSelection({ col: 1, row: 0 }, "l", groups), { col: 2, row: 0 });
  assert.deepEqual(moveSelection({ col: 0, row: 0 }, "ArrowLeft", groups), { col: 0, row: 0 });
  assert.equal(moveSelection({ col: 0, row: 0 }, "x", groups), null);
  assert.deepEqual(clampSelection({ col: 0, row: 5 }, groups), { col: 0, row: 2 });
});

test("locates a task by id", () => {
  const groups = groupTasks([
    task("a", "today", "2026-10-18T00:00:00Z"),
    task("b", "today", "2026-10-18T01:00:00Z"),
  ]);
  assert.deepEqual(locate("a", groups), { col: 1, row: 1 });
  assert.equal(locate("zzz", groups), null);
});
