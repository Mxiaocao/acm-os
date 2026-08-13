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
      await waitFor(() => [...document.querySelectorAll("button, a")].some(matches), value);
      [...document.querySelectorAll("button, a")].find(matches).click();
    };
    const inputValue = (input, value) => {
      if (!input) throw new Error(`Missing input for ${value}`);
      Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value").set.call(input, value);
      input.dispatchEvent(new Event("input", { bubbles: true }));
      input.dispatchEvent(new Event("change", { bubbles: true }));
    };
    const setDate = (localDate) => invoke("desktop_e2e_set_date", { input: { localDate } });
    const stage = (value) => invoke("desktop_e2e_log", { input: { stage: value } });
    const budgetInput = (day) => document.querySelector(`input[aria-label="${day} ACM budget in minutes"]`);
    const summaryValue = (label) => {
      const term = [...document.querySelectorAll(".today-toolbar dt")].find((item) => item.textContent.trim() === label);
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
    const navigateTo = async (label, readyText = label) => {
      await clickText(label);
      await waitText(readyText);
    };

    try {
      await stage("driver-started");
      const configured = await invoke("desktop_e2e_context");
      if (configured.phase === "verify-restart") {
        await waitText("Today");
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
        return;
      }

      await waitText("Connect your workspace");
      await stage("workspace-shell-ready");
      const inputs = [...document.querySelectorAll(".workspace-form input")];
      [configured.vault, configured.problems, configured.knowledge]
        .forEach((value, index) => inputValue(inputs[index], value));
      document.querySelector(".workspace-form button[type=submit]").click();

      await waitText("Today");
      await stage("workspace-configured");

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
      await clickText("Preview replan");
      await waitText("Apply this replan?");
      assertText(summaryValue("Budget"), "73 min", "Preview must not mutate Today");
      await clickText("Cancel");
      await waitFor(() => !bodyText().includes("Apply this replan?"), "cancelled preview");
      assertText(summaryValue("Budget"), "73 min", "Cancel must not persist");
      inputValue(document.querySelector('input[aria-label="Daily budget in minutes"]'), "47");
      await clickText("Preview replan");
      await waitText("Apply this replan?");
      await clickText("Apply replan");
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

      await clickText("我的题库");
      await clickText("A. Desktop E2E Problem");
      await clickText("Create my note");
      await waitText("Personal Markdown created and verified.");
      await clickText("加入补题");
      await waitText("开始学习");
      await clickText("开始学习");
      await waitText("我已经补懂");
      await clickText("我已经补懂");
      await waitText("Next Review due: 2026-08-14");
      await stage("problem-a-learned");

      await clickText("我的题库");
      await clickText("B. Desktop E2E Study Problem");
      await clickText("Create my note");
      await waitText("Personal Markdown created and verified.");
      await clickText("加入补题");
      await waitText("开始学习");
      await stage("problem-c-queued");

      await clickText("我的题库");
      await clickText("C. Desktop E2E Extra Study Problem");
      await clickText("Create my note");
      await waitText("Personal Markdown created and verified.");
      await clickText("加入补题");
      await waitText("开始学习");

      await setDate("2026-08-14");
      await clickText("我的题库");
      await clickText("A. Desktop E2E Problem");
      await waitText("Start Review");
      await clickText("Start Review");
      await waitText("Finish this Review");
      await clickText("Complete from facts");
      await waitText("Mastered");
      await stage("review-completed");

      await setDate("2026-08-24");
      await clickText("Return to Today");
      await waitFor(() => summaryValue("Budget") === "95 min", "Monday weekly default");
      inputValue(document.querySelector('input[aria-label="Daily budget in minutes"]'), "180");
      await clickText("Preview replan");
      await waitText("Apply this replan?");
      await clickText("Apply replan");
      await waitFor(() => summaryValue("Budget") === "180 min", "applied Monday override");
      await waitText("Long-term Review");
      await waitText("Upsolve");
      await stage("today-generated");
      if (!/Review[\s\S]*Desktop E2E Problem[\s\S]*Long-term Review/.test(bodyText())) {
        throw new Error("Later Today did not contain the authoritative Review recall");
      }

      await invoke("desktop_e2e_finish", { input: { result: "restart" } });
    } catch (error) {
      await invoke("desktop_e2e_finish", {
        input: { result: `failed-${String(error).slice(0, 1500)}` },
      });
    }
  }, 250);
}
