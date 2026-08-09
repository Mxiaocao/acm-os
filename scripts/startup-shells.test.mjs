import assert from "node:assert/strict";
import test from "node:test";
import { parseAppRoute } from "../src/app/routing.ts";

test("maps the frozen normal application routes", () => {
  assert.deepEqual(parseAppRoute("/"), { kind: "normal", page: "today" });
  assert.deepEqual(parseAppRoute("/today"), { kind: "normal", page: "today" });
  assert.deepEqual(parseAppRoute("/contests/"), { kind: "normal", page: "contests" });
  assert.deepEqual(parseAppRoute("problems"), { kind: "normal", page: "problems" });
  assert.deepEqual(parseAppRoute("/knowledge"), { kind: "normal", page: "knowledge" });
  assert.deepEqual(parseAppRoute("/settings"), { kind: "normal", page: "settings" });
});

test("recognizes one stable Review Attempt id without exposing normal navigation", () => {
  const attemptId = "018f0d8e-4a5b-7c6d-8e9f-0123456789ab";
  assert.deepEqual(parseAppRoute(`/review/${attemptId}`), {
    kind: "review",
    attemptId,
  });
});

test("rejects incomplete, nested, and malformed Review routes", () => {
  assert.deepEqual(parseAppRoute("/review"), { kind: "notFound", pathname: "/review" });
  assert.deepEqual(parseAppRoute("/review/a/b"), {
    kind: "notFound",
    pathname: "/review/a/b",
  });
  assert.deepEqual(parseAppRoute("/review/%2F"), {
    kind: "notFound",
    pathname: "/review/%2F",
  });
  assert.deepEqual(parseAppRoute("/review/%E0%A4%A"), {
    kind: "notFound",
    pathname: "/review/%E0%A4%A",
  });
  for (const value of [
    "018f0d8e-4a5b-6c6d-8e9f-0123456789ab",
    "018f0d8e-4a5b-7c6d-7e9f-0123456789ab",
    "018f0d8e%2D4a5b%2D7c6d%2D8e9f%2D0123456789ab",
  ]) {
    assert.equal(parseAppRoute(`/review/${value}`).kind, "notFound");
  }
});

test("does not let setup or recovery URLs masquerade as normal pages", () => {
  assert.deepEqual(parseAppRoute("/setup"), { kind: "notFound", pathname: "/setup" });
  assert.deepEqual(parseAppRoute("/recovery"), {
    kind: "notFound",
    pathname: "/recovery",
  });
});
