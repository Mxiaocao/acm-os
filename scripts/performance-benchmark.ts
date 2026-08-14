import { performance } from "node:perf_hooks";
import { parseAppRoute } from "../src/app/routing.ts";

const REFERENCE = {
  problems: 2_000,
  markdown: 1_000,
  contests: 300,
  attempts: 10_000,
  knowledge: 1_000,
  relations: 20_000,
} as const;

const BUDGETS_MS = {
  startupProjection: 2_500,
  todayOpen: 300,
  knowledgeSearch: 150,
  markdownParse: 500,
  relationProjection: 300,
  localNavigation: 300,
} as const;

type Problem = { id: number; status: "ready" | "due" | "completed"; cost: 30 | 60 };
type Knowledge = { id: number; title: string; body: string };
type Relation = { from: number; to: number };

const problems: Problem[] = Array.from({ length: REFERENCE.problems }, (_, id) => ({
  id,
  status: id % 7 === 0 ? "due" : id % 5 === 0 ? "completed" : "ready",
  cost: id % 3 === 0 ? 60 : 30,
}));
const markdown = Array.from({ length: REFERENCE.markdown }, (_, id) =>
  `# Problem ${id}\n\n## 题解\n\n${"content ".repeat(24)}\n\n## 额外题目\n\n- route-${id}`,
);
const knowledge: Knowledge[] = Array.from({ length: REFERENCE.knowledge }, (_, id) => ({
  id,
  title: `Knowledge Node ${id}`,
  body: `concept-${id % 100} ${"detail ".repeat(10)}`,
}));
const relations: Relation[] = Array.from({ length: REFERENCE.relations }, (_, id) => ({
  from: id % REFERENCE.knowledge,
  to: (id * 17) % REFERENCE.knowledge,
}));
const routes = [
  "/today",
  "/contests/1979",
  "/problems/1979/A",
  "/knowledge",
  "/review/018f0d8e-4a5b-7c6d-8e9f-0123456789ab",
];

function percentile95(samples: number[]): number {
  const sorted = [...samples].sort((left, right) => left - right);
  return sorted[Math.min(sorted.length - 1, Math.ceil(sorted.length * 0.95) - 1)];
}

function measure(name: string, budgetMs: number, operation: () => void): { name: string; p95Ms: number; budgetMs: number } {
  for (let index = 0; index < 5; index += 1) operation();
  const samples: number[] = [];
  for (let index = 0; index < 25; index += 1) {
    const start = performance.now();
    operation();
    samples.push(performance.now() - start);
  }
  return { name, p95Ms: percentile95(samples), budgetMs };
}

const results = [
  measure("startupProjection", BUDGETS_MS.startupProjection, () => {
    const attemptState = new Map<number, string>();
    for (let id = 0; id < REFERENCE.attempts; id += 1) attemptState.set(id, id % 3 === 0 ? "due" : "waiting");
    const relationState = new Map<number, number>();
    for (const relation of relations) relationState.set(relation.from, (relationState.get(relation.from) ?? 0) + 1);
    if (attemptState.size !== REFERENCE.attempts || relationState.size === 0) throw new Error("startup projection benchmark corrupted");
  }),
  measure("todayOpen", BUDGETS_MS.todayOpen, () => {
    const plan = problems
      .filter((problem) => problem.status !== "completed")
      .sort((left, right) => left.cost - right.cost || left.id - right.id)
      .slice(0, 120);
    if (plan.length !== 120) throw new Error("Today benchmark corrupted");
  }),
  measure("knowledgeSearch", BUDGETS_MS.knowledgeSearch, () => {
    const matches = knowledge.filter((node) => `${node.title} ${node.body}`.toLowerCase().includes("concept-42"));
    if (matches.length === 0) throw new Error("Knowledge search benchmark corrupted");
  }),
  measure("markdownParse", BUDGETS_MS.markdownParse, () => {
    let sections = 0;
    for (const document of markdown) sections += (document.match(/^## /gm) ?? []).length;
    if (sections !== REFERENCE.markdown * 2) throw new Error("Markdown benchmark corrupted");
  }),
  measure("relationProjection", BUDGETS_MS.relationProjection, () => {
    const outgoing = new Map<number, number[]>();
    for (const relation of relations) {
      const current = outgoing.get(relation.from) ?? [];
      current.push(relation.to);
      outgoing.set(relation.from, current);
    }
    if (outgoing.size !== REFERENCE.knowledge) throw new Error("Relation benchmark corrupted");
  }),
  measure("localNavigation", BUDGETS_MS.localNavigation, () => {
    for (let index = 0; index < 1_000; index += 1) parseAppRoute(routes[index % routes.length]);
  }),
];

console.log(JSON.stringify({ reference: REFERENCE, budgetsMs: BUDGETS_MS, results }, null, 2));
const failures = results.filter((result) => result.p95Ms > result.budgetMs);
if (failures.length > 0) {
  throw new Error(`Performance budget exceeded: ${failures.map((result) => `${result.name} p95=${result.p95Ms.toFixed(2)}ms > ${result.budgetMs}ms`).join(", ")}`);
}
