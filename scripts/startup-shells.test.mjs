import assert from "node:assert/strict";
import test from "node:test";
import { parseAppRoute } from "../src/app/routing.ts";

test("maps the frozen normal application routes", () => {
  assert.deepEqual(parseAppRoute("/"), { kind: "normal", page: "today" });
  assert.deepEqual(parseAppRoute("/today"), { kind: "normal", page: "today" });
  assert.deepEqual(parseAppRoute("/contests/"), { kind: "normal", page: "contests" });
  assert.deepEqual(parseAppRoute("problems"), { kind: "normal", page: "problems" });
  assert.deepEqual(parseAppRoute("/knowledge"), { kind: "normal", page: "knowledge" });
  assert.deepEqual(parseAppRoute("/reward"), { kind: "normal", page: "reward" });
  assert.deepEqual(parseAppRoute("/settings"), { kind: "normal", page: "settings" });
});

test("accepts only canonical M1 problem-detail identities", () => {
  assert.deepEqual(parseAppRoute("/problems/1979/A"), {
    kind: "problemDetail", contestId: 1979, index: "A",
  });
  assert.deepEqual(parseAppRoute("/problems/1/F1"), {
    kind: "problemDetail", contestId: 1, index: "F1",
  });
  for (const pathname of [
    "/problems/0/A", "/problems/01/A", "/problems/1979/a",
    "/problems/1979/%41", "/problems/1979/A/extra", "/problems/1979/A%2F",
  ]) {
    assert.equal(parseAppRoute(pathname).kind, "notFound");
  }
});

test("accepts only canonical M1 contest-detail identities", () => {
  assert.deepEqual(parseAppRoute("/contests/1979"), {
    kind: "contestDetail", contestId: 1979,
  });
  for (const pathname of ["/contests/0", "/contests/01", "/contests/1979/A", "/contests/%31"]) {
    assert.equal(parseAppRoute(pathname).kind, "notFound");
  }
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
