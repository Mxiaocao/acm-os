import { ArrowRight, CalendarDots, ChartBar, CheckCircle, Clock, Notebook, Play, Timer } from "@phosphor-icons/react";
import type { TodaySnapshotDto } from "../ipc/today";

export function TodayOverview({ snapshot, navigate }: { snapshot: TodaySnapshotDto | null; navigate: (path: string) => void }) {
  const entries = snapshot?.entries ?? [];
  const completed = entries.filter((entry) => entry.status === "completed").length;
  const active = entries.filter((entry) => entry.status === "inProgress").length;
  const available = entries.filter((entry) => entry.status !== "completed" && entry.status !== "unavailable");
  const reviewMinutes = entries.filter((entry) => entry.lane === "review" || entry.reason === "continueReview").reduce((sum, entry) => sum + entry.planningCostMinutes, 0);
  const studyMinutes = (snapshot?.plannedMinutes ?? 0) - reviewMinutes;
  const next = available[0];
  return <aside className="today-overview" aria-label="今日学习概览">
    <section className="today-overview__panel">
      <header><h2><ChartBar size={18} aria-hidden="true" />今日概览</h2><span>{snapshot?.localDate ?? "读取中"}</span></header>
      {snapshot ? <dl className="today-metrics">
        <div><Notebook size={19} aria-hidden="true" /><dt>计划任务</dt><dd>{entries.length}<small>项</small></dd></div>
        <div><CheckCircle size={19} aria-hidden="true" /><dt>已完成</dt><dd>{completed}<small>项</small></dd></div>
        <div><Play size={19} aria-hidden="true" /><dt>进行中</dt><dd>{active}<small>项</small></dd></div>
        <div><CalendarDots size={19} aria-hidden="true" /><dt>待处理</dt><dd>{available.length}<small>项</small></dd></div>
        <div><Clock size={19} aria-hidden="true" /><dt>计划用时</dt><dd>{snapshot.plannedMinutes}<small>分钟</small></dd></div>
        <div><Timer size={19} aria-hidden="true" /><dt>剩余预算</dt><dd>{Math.max(0, snapshot.budgetMinutes - snapshot.plannedMinutes)}<small>分钟</small></dd></div>
      </dl> : <p className="today-overview__empty">设置今日预算后，这里会显示计划概览。</p>}
    </section>
    <section className="today-overview__panel">
      <header><h2>计划构成</h2><span>按预计用时</span></header>
      <div className="today-composition" aria-label={`学习 ${studyMinutes} 分钟，复习 ${reviewMinutes} 分钟`}>
        <div className="today-composition__track" aria-hidden="true">
          <span style={{ width: `${snapshot?.plannedMinutes ? studyMinutes / snapshot.plannedMinutes * 100 : 0}%` }} />
          <span style={{ width: `${snapshot?.plannedMinutes ? reviewMinutes / snapshot.plannedMinutes * 100 : 0}%` }} />
        </div>
        <dl><div><dt>学习 / 补题</dt><dd>{studyMinutes} 分钟</dd></div><div><dt>冷启动 / 复习</dt><dd>{reviewMinutes} 分钟</dd></div></dl>
      </div>
    </section>
    <section className="today-overview__panel today-next-step">
      <header><h2>接下来</h2></header>
      {next ? <><p>{next.problemTitle}</p><span>{next.planningCostMinutes} 分钟 · {next.reason === "continueReview" || next.lane === "review" ? "复习任务" : "学习任务"}</span>
        <button className="primary-action" onClick={() => navigate(next.reviewAttemptId && next.status === "inProgress" ? `/review/${next.reviewAttemptId}` : `/problems/id/${next.problemId}`)} type="button"><Play size={15} weight="fill" aria-hidden="true" />继续学习</button></>
        : <><p>给下一次进步留一点时间。</p><span>{snapshot?.entries.length ? "今日计划已没有可执行的任务。" : "从题库选题，或查看接下来的复习。"}</span>
          <button className="secondary-action" onClick={() => navigate("/problems")} type="button">打开我的题库<ArrowRight size={15} aria-hidden="true" /></button></>}
      <button className="today-overview__calendar" onClick={() => navigate("/calendar")} type="button"><CalendarDots size={16} aria-hidden="true" />查看学习月历<ArrowRight size={14} aria-hidden="true" /></button>
    </section>
  </aside>;
}
