import { invoke } from "@tauri-apps/api/core";

if (!window.__ACM_OS_DESKTOP_E2E_DRIVER__) {
  window.__ACM_OS_DESKTOP_E2E_DRIVER__ = true;
  setTimeout(async () => {
    const delay = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
    const bodyText = () => document.body.innerText;
    const waitFor = async (predicate, label, timeout = 15000) => {
      const deadline = Date.now() + timeout;
      while (Date.now() < deadline) {
        if (predicate()) return;
        await delay(100);
      }
      throw new Error(`Timed out waiting for ${label}: ${bodyText()}`);
    };
    const waitText = (value) => waitFor(() => bodyText().includes(value), value);
    const clickText = async (value) => {
      const matches = (item) => item.textContent.trim().startsWith(value);
      await waitFor(() => {
        const item = [...document.querySelectorAll("button, a")].find(matches);
        if (!item) return false;
        item.click();
        return true;
      }, value);
    };
    const lifecycleSection = () => document.querySelector('section[aria-labelledby="learning-lifecycle-heading"]');
    const lifecyclePrimary = () => lifecycleSection()?.querySelector(".action-row button.primary-action");
    const advanceLifecycle = async (label) => {
      let action;
      await waitFor(() => {
        action = lifecyclePrimary();
        return action != null;
      }, `${label} lifecycle action`);
      action.click();
      await waitFor(() => !document.contains(action), `${label} lifecycle transition`);
    };
    const assertNextReviewDate = async (expected) => {
      await waitFor(() => {
        const text = lifecycleSection()?.textContent ?? "";
        return text.match(/\b\d{4}-\d{2}-\d{2}\b/)?.[0] === expected;
      }, `next Review date ${expected}`);
    };
    const startReview = async () => {
      let action;
      await waitFor(() => {
        action = lifecyclePrimary();
        return action != null;
      }, "Review action");
      action.click();
      await waitFor(() => window.location.pathname.startsWith("/review/"), "Review route");
      await waitFor(() => document.querySelector("form.review-facts-form") !== null, "Review facts form");
    };
    const completeReview = async () => {
      let completionControl;
      await waitFor(() => {
        completionControl = document.querySelector("form.review-facts-form button[type=submit]");
        return completionControl !== null;
      }, "Review completion control");
      completionControl.click();
      await waitFor(() => {
        const card = document.querySelector('section.review-evidence-card[aria-labelledby="review-evidence-title"]');
        if (!card) return false;
        return card.querySelector("h2")?.textContent.trim() !== ""
          && card.querySelectorAll("h3").length >= 2
          && card.querySelector("ul") !== null;
      }, "completed Review evidence");
    };
    const returnToToday = async () => {
      let returnControl;
      await waitFor(() => {
        returnControl = document.querySelector('main.review-shell header button.secondary-action');
        return returnControl !== null;
      }, "Return to Today control");
      returnControl.click();
      await waitFor(() => window.location.pathname === "/today", "Today route after Review");
    };
    const assertKnowledgeReevaluation = async (targetRef, expectedLevel, expectedCount) => {
      await waitFor(() => {
        const detail = [...document.querySelectorAll("section.knowledge-detail")]
          .find((section) => section.querySelector("#knowledge-detail-title")?.textContent.trim() === targetRef);
        const understanding = detail?.querySelector(".knowledge-understanding select");
        const suggestion = detail?.querySelector(".knowledge-understanding > p.safe-note");
        return understanding?.value === expectedLevel
          && suggestion?.textContent.match(/\d+/)?.[0] === String(expectedCount);
      }, `${targetRef} Knowledge reevaluation suggestion for ${expectedCount} qualifying Problems`);
    };
    const inputValue = (input, value) => {
      if (!input) throw new Error(`Missing input for ${value}`);
      Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value").set.call(input, value);
      input.dispatchEvent(new Event("input", { bubbles: true }));
      input.dispatchEvent(new Event("change", { bubbles: true }));
    };
    const selectValue = (select, value) => {
      if (!select) throw new Error(`Missing select for ${value}`);
      Object.getOwnPropertyDescriptor(HTMLSelectElement.prototype, "value").set.call(select, value);
      select.dispatchEvent(new Event("change", { bubbles: true }));
    };
    const setDate = (localDate) => invoke("desktop_e2e_set_date", { input: { localDate } });
    const stage = (value) => invoke("desktop_e2e_log", { input: { stage: value } });
    const budgetInput = (day) => document.querySelector(`input[aria-label="${day} ACM budget in minutes"]`);
    const summaryValue = (label) => {
      const index = { Date: 0, Planned: 1, Budget: 2, Over: 3 }[label];
      const term = document.querySelectorAll(".today-toolbar dt")[index];
      return term?.nextElementSibling?.textContent.trim() ?? "";
    };
    const assertText = (actual, expected, label) => {
      if (actual !== expected) throw new Error(`${label}: expected ${expected}, got ${actual}`);
    };
    const assertWeeklyInputs = (expected) => {
      for (const [day, value] of Object.entries(expected)) {
        assertText(budgetInput(day)?.value ?? "<missing>", value, `${day} weekly budget`);
      }
    };
    const createPersonalNote = async () => {
      await waitFor(
        () => document.querySelector('section.content-panel button.primary-action[type="button"]') !== null,
        "Personal Markdown create control",
      );
      document.querySelector('section.content-panel button.primary-action[type="button"]').click();
      await waitFor(
        () => document.querySelector('section.content-panel[aria-label="Personal Markdown projection"]') !== null,
        "Personal Markdown projection",
      );
      await waitFor(
        () => document.querySelector('section.content-panel[aria-label="Personal Markdown projection"] code')?.textContent.trim() !== "",
        "Personal Markdown binding",
      );
    };
    const knowledgeCandidateRow = (targetRef) => [...document.querySelectorAll("section.knowledge-candidates > ul > li")]
      .find((item) => item.querySelector("strong")?.textContent.trim() === targetRef);
    const knowledgeCandidateAction = (targetRef, selector) => knowledgeCandidateRow(targetRef)?.querySelector(selector);
    const saveKnowledgeIntent = async (targetRef) => {
      await waitFor(
        () => knowledgeCandidateAction(targetRef, "button:not(.secondary-action)") != null,
        "Knowledge intent action",
      );
      knowledgeCandidateAction(targetRef, "button:not(.secondary-action)").click();
      await waitFor(
        () => {
          const row = knowledgeCandidateRow(targetRef);
          return row
            && row.querySelector("button:not(.secondary-action)") === null
            && row.querySelectorAll("button.secondary-action").length === 2;
        },
        "accepted Knowledge intent",
      );
    };
    const ignoreKnowledgeCandidate = async (targetRef) => {
      await waitFor(
        () => knowledgeCandidateAction(targetRef, "button.secondary-action") != null,
        "Knowledge candidate ignore action",
      );
      knowledgeCandidateAction(targetRef, "button.secondary-action").click();
      await waitFor(
        () => {
          const row = knowledgeCandidateRow(targetRef);
          return row
            && row.querySelector("button:not(.secondary-action)") === null
            && row.querySelectorAll("button.secondary-action").length === 1;
        },
        "ignored Knowledge candidate",
      );
    };
    const acceptKnowledgeCandidate = async (targetRef) => {
      await waitFor(
        () => knowledgeCandidateAction(targetRef, "button:not(.secondary-action)") != null,
        "pending Knowledge candidate",
      );
      knowledgeCandidateAction(targetRef, "button:not(.secondary-action)").click();
      await waitFor(
        () => knowledgeCandidateRow(targetRef) === undefined
          && document.querySelector('section.content-panel[aria-label="Personal Markdown projection"]') !== null,
        "accepted Knowledge relation",
      );
    };
    const navigateTo = async (label, readyText = label) => {
      const primaryRoute = {
        Today: "/today",
        Contests: "/contests",
        Problems: "/problems",
        Knowledge: "/knowledge",
        Settings: "/settings",
      }[label];
      if (primaryRoute) {
        let routeControl;
        await waitFor(
          () => {
            routeControl = document.querySelector(`nav a[href="${primaryRoute}"]`);
            return routeControl !== null;
          },
          `${label} primary route`,
        );
        routeControl.click();
        await waitFor(() => window.location.pathname === primaryRoute, `${label} route`);
      } else {
        await clickText(label);
        await waitText(readyText);
      }
    };

    try {
      await stage("driver-started");
      const configured = await invoke("desktop_e2e_context");
      if (configured.phase === "verify-restart") {
        await waitFor(
          () => document.querySelector('nav a[href="/today"]') !== null,
          "primary Today route after restart",
        );
        await navigateTo("Knowledge", "Knowledge index");
        await clickText("Segment Tree");
        await waitFor(() => document.querySelector(".knowledge-understanding select") !== null, "persisted Knowledge detail");
        await waitFor(() => document.querySelector(".knowledge-understanding > p strong") !== null, "persisted Knowledge understanding");
        assertText(document.querySelector(".knowledge-understanding select")?.value ?? "<missing>", "basic", "persisted Knowledge understanding");
        await stage("restart-knowledge-restored");

        await clickText("我的题库");
        await clickText("A. Desktop E2E Problem");
        await waitText("Segment Tree Candidate");
        await waitText("Ignored");
        await waitText("Fenwick Tree Intent");
        await waitText("Accepted intent · existing Knowledge Markdown now found");
        await clickText("Accept existing Knowledge");
        await waitText("verified as a formal relation");
        await stage("restart-candidate-ignored-restored");
        await stage("restart-accepted-intent-explicitly-safe-patched");

        await navigateTo("Knowledge", "Knowledge index");
        await clickText("Segment Tree");
        await waitText("1979A · Desktop E2E Problem");
        await waitText("Consider re-evaluating this Knowledge status");
        await waitText("3 distinct related Problems gained new 真会 Review Evidence");
        await stage("restart-safe-patch-relation-restored");
        await stage("restart-reevaluation-suggestion-restored");

        await navigateTo("Today");
        await waitFor(() => summaryValue("Budget") === "180 min", "persisted 180 minute override");
        await stage("restart-today-restored");

        await navigateTo("Settings", "Weekly ACM budget");
        await waitFor(() => budgetInput("Monday")?.value === "95", "persisted weekly settings");
        assertWeeklyInputs({ Monday: "95", Tuesday: "73", Wednesday: "", Thursday: "101", Friday: "47", Saturday: "0", Sunday: "95" });
        await stage("restart-weekly-restored");

        await setDate("2026-08-31");
        await navigateTo("Today");
        await waitFor(() => summaryValue("Date") === "2026-08-31", "next Monday Today plan");
        assertText(summaryValue("Budget"), "95 min", "next Monday weekly default");
        await stage("next-week-default-restored");
        await invoke("desktop_e2e_finish", { input: { result: "passed" } });
        await invoke("desktop_e2e_exit");
        return;
      }

      await waitFor(
        () => document.querySelector(".workspace-form button[type=submit]") !== null,
        "workspace setup form",
      );
      await stage("workspace-shell-ready");
      const inputs = [...document.querySelectorAll(".workspace-form input")];
      [configured.vault, configured.problems, configured.knowledge]
        .forEach((value, index) => inputValue(inputs[index], value));
      document.querySelector(".workspace-form button[type=submit]").click();

      await waitFor(
          () => document.querySelector('nav a[href="/today"]') !== null,
        "primary Today route",
      );
      await stage("workspace-configured");

      await navigateTo("Knowledge", "Knowledge index");
      await waitText("Segment Tree");
      await clickText("Segment Tree");
      await waitFor(() => document.querySelector(".knowledge-understanding select") !== null, "Knowledge detail");
      await waitFor(
        () => document.querySelector(".knowledge-understanding > p strong") === null,
        "unconfirmed Knowledge understanding",
      );
      selectValue(document.querySelector(".knowledge-understanding select"), "basic");
      document.querySelector(".knowledge-understanding > button").click();
      await waitFor(() => document.querySelector(".knowledge-understanding > p strong") !== null, "confirmed Knowledge understanding");
      assertText(document.querySelector(".knowledge-understanding select")?.value ?? "<missing>", "basic", "confirmed Knowledge understanding");
      await stage("knowledge-confirmed");

      await navigateTo("Settings", "Weekly ACM budget");
      await waitFor(() => budgetInput("Monday") !== null, "weekly budget inputs");
      const weekly = { Monday: "95", Tuesday: "73", Wednesday: "", Thursday: "101", Friday: "47", Saturday: "0", Sunday: "95" };
      for (const [day, value] of Object.entries(weekly)) inputValue(budgetInput(day), value);
      await clickText("Save weekly budget");
      await waitText("Weekly ACM budget saved.");
      await stage("weekly-saved");

      await navigateTo("Today");
      await navigateTo("Settings", "Weekly ACM budget");
      await waitFor(() => budgetInput("Tuesday")?.value === "73", "reopened weekly settings");
      assertWeeklyInputs(weekly);

      let rejected = false;
      try {
        await invoke("save_weekly_acm_budget", { schedule: {
          monday: -1, tuesday: 999, wednesday: 999, thursday: 999,
          friday: 999, saturday: 999, sunday: 999,
        } });
      } catch {
        rejected = true;
      }
      if (!rejected) throw new Error("Invalid weekly budget IPC was accepted");
      await navigateTo("Today");
      await navigateTo("Settings", "Weekly ACM budget");
      await waitFor(() => budgetInput("Monday")?.value === "95", "weekly settings after invalid write");
      assertWeeklyInputs(weekly);
      await stage("weekly-invalid-atomic");

      await navigateTo("Today");
      await waitFor(() => summaryValue("Date") === "2026-08-11", "Tuesday Today plan");
      assertText(summaryValue("Budget"), "73 min", "Tuesday weekly default");
      const override = document.querySelector('input[aria-label="Daily budget in minutes"]');
      inputValue(override, "47");
      document.querySelector(".today-toolbar button[type=submit]").click();
      await waitFor(() => document.querySelector('[role="dialog"][aria-labelledby="today-replan-title"]') !== null, "replan dialog");
      assertText(summaryValue("Budget"), "73 min", "Preview must not mutate Today");
      document.querySelector('[role="dialog"] button.secondary-action').click();
      await waitFor(() => document.querySelector('[role="dialog"][aria-labelledby="today-replan-title"]') === null, "cancelled preview");
      assertText(summaryValue("Budget"), "73 min", "Cancel must not persist");
      inputValue(document.querySelector('input[aria-label="Daily budget in minutes"]'), "47");
      document.querySelector(".today-toolbar button[type=submit]").click();
      await waitFor(() => document.querySelector('[role="dialog"][aria-labelledby="today-replan-title"]') !== null, "replan dialog");
      document.querySelector('[role="dialog"] button.primary-action').click();
      await waitFor(() => summaryValue("Budget") === "47 min", "applied 47 minute override");
      await navigateTo("Settings", "Weekly ACM budget");
      await waitFor(() => budgetInput("Tuesday")?.value === "73", "weekly template after override");
      assertWeeklyInputs(weekly);
      await stage("date-local-override-verified");

      await navigateTo("Contests", "Codeforces 公开比赛网址");
      const contest = document.querySelector('input[placeholder*="codeforces.com/contest"]');
      inputValue(contest, "https://codeforces.com/contest/1979");
      contest.form.requestSubmit();
      await waitText("Desktop E2E Contest");
      await stage("contest-imported");

      const importedBook = document.querySelector("button.contest-book");
      if (!importedBook) throw new Error("Imported ContestBook was not rendered as a native button");
      if (importedBook.type !== "button") throw new Error(`ContestBook type was ${importedBook.type}`);
      if (!importedBook.getAttribute("aria-label")?.includes("Desktop E2E Contest")) {
        throw new Error(`ContestBook accessible name did not retain its Contest identity: ${importedBook.getAttribute("aria-label")}`);
      }
      importedBook.click();
      await waitFor(() => window.location.pathname === "/contests/1979", "ContestBook detail navigation");
      await waitText("Desktop E2E Contest");
      await waitFor(
        () => document.activeElement === document.querySelector("h1"),
        "focused Contest Detail heading",
      );
      await stage("contest-book-navigation-and-detail-focus-verified");

      await clickText("我的题库");
      await clickText("A. Desktop E2E Problem");
      await createPersonalNote();
      await invoke("register_knowledge_candidate", { input: {
        contestId: 1979,
        index: "A",
        fingerprint: "de".repeat(32),
        targetRef: "Fenwick Tree Intent",
      } });
      await clickText("我的题库");
      await clickText("A. Desktop E2E Problem");
      await waitFor(() => knowledgeCandidateRow("Fenwick Tree Intent") !== undefined, "Fenwick Tree intent candidate");
      await saveKnowledgeIntent("Fenwick Tree Intent");
      await stage("candidate-accepted-intent-without-authority");
      await invoke("register_knowledge_candidate", { input: {
        contestId: 1979,
        index: "A",
        fingerprint: "ab".repeat(32),
        targetRef: "Segment Tree Candidate",
      } });
      await clickText("我的题库");
      await clickText("A. Desktop E2E Problem");
      await waitFor(() => knowledgeCandidateRow("Segment Tree Candidate") !== undefined, "Segment Tree candidate");
      await ignoreKnowledgeCandidate("Segment Tree Candidate");
      await stage("candidate-ignored-without-authority");
      await invoke("register_knowledge_candidate", { input: {
        contestId: 1979,
        index: "A",
        fingerprint: "bc".repeat(32),
        targetRef: "Segment Tree",
      } });
      await clickText("我的题库");
      await clickText("A. Desktop E2E Problem");
      await acceptKnowledgeCandidate("Segment Tree");
      await stage("candidate-safe-patched");
      await advanceLifecycle("join upsolve");
      await advanceLifecycle("start learning");
      await advanceLifecycle("mark understood");
      await assertNextReviewDate("2026-08-14");
      await stage("problem-a-learned");

      await navigateTo("Problems");
      await clickText("B. Desktop E2E Study Problem");
      await createPersonalNote();
      await invoke("register_knowledge_candidate", { input: { contestId: 1979, index: "B", fingerprint: "be".repeat(32), targetRef: "Segment Tree" } });
      await navigateTo("Problems");
      await clickText("B. Desktop E2E Study Problem");
      await acceptKnowledgeCandidate("Segment Tree");
      await advanceLifecycle("join upsolve");
      await advanceLifecycle("start learning");
      await advanceLifecycle("mark understood");
      await assertNextReviewDate("2026-08-14");
      await stage("problem-b-learned");

      await navigateTo("Problems");
      await clickText("C. Desktop E2E Extra Study Problem");
      await createPersonalNote();
      await invoke("register_knowledge_candidate", { input: { contestId: 1979, index: "C", fingerprint: "ce".repeat(32), targetRef: "Segment Tree" } });
      await navigateTo("Problems");
      await clickText("C. Desktop E2E Extra Study Problem");
      await acceptKnowledgeCandidate("Segment Tree");
      await advanceLifecycle("join upsolve");
      await advanceLifecycle("start learning");
      await advanceLifecycle("mark understood");
      await assertNextReviewDate("2026-08-14");
      await stage("problem-c-learned");

      await setDate("2026-08-14");
      await navigateTo("Problems");
      await clickText("A. Desktop E2E Problem");
      await startReview();
      await completeReview();
      await stage("review-a-completed");

      await returnToToday();
      await navigateTo("Problems");
      await clickText("B. Desktop E2E Study Problem");
      await startReview();
      await completeReview();
      await stage("review-b-completed");

      await returnToToday();
      await navigateTo("Problems");
      await clickText("C. Desktop E2E Extra Study Problem");
      await startReview();
      await completeReview();
      await stage("review-c-completed");

      await returnToToday();
      await navigateTo("Knowledge", "Knowledge index");
      await clickText("Segment Tree");
      await assertKnowledgeReevaluation("Segment Tree", "basic", 3);
      await stage("reevaluation-suggestion-visible");

      await setDate("2026-08-24");
      await navigateTo("Today");
      await waitFor(() => summaryValue("Budget") === "95 min", "Monday weekly default");
      inputValue(document.querySelector('input[aria-label="Daily budget in minutes"]'), "180");
      document.querySelector(".today-toolbar button[type=submit]").click();
      await waitFor(() => document.querySelector('[role="dialog"][aria-labelledby="today-replan-title"]') !== null, "replan dialog");
      document.querySelector('[role="dialog"] button.primary-action').click();
      await waitFor(() => summaryValue("Budget") === "180 min", "applied Monday override");
      await waitText("Long-term Review");
      await stage("today-generated");
      if (!/Review[\s\S]*Desktop E2E Problem[\s\S]*Long-term Review/.test(bodyText())) {
        throw new Error("Later Today did not contain the authoritative Review recall");
      }
      if ((bodyText().match(/Long-term Review/g) ?? []).length !== 3) {
        throw new Error("Later Today did not contain all three authoritative Review recalls");
      }

      await invoke("desktop_e2e_finish", { input: { result: "restart" } });
      await invoke("desktop_e2e_exit");
    } catch (error) {
      await invoke("desktop_e2e_finish", {
        input: { result: `failed-${String(error).slice(0, 1500)}` },
      });
      await invoke("desktop_e2e_exit");
    }
  }, 250);
}
