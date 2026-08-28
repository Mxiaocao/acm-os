const RUSSIAN_PROBLEM_TITLES: Record<string, string> = {
  "Три числа на доске": "Three Numbers on the Blackboard",
  "Плитки домино": "Domino Tiles",
  "Горячая картошка на складе фей": "Hot Potatoes at the Fairy Warehouse",
  "Лента для завтрашнего дня": "A Ribbon for Tomorrow",
  "Даже если весь мир перевернётся": "Even If the World Turns",
  "Сколько времени пройдет, пока ничего не останется?": "How Long Until Nothing Remains?",
  "Сколько времени пройдёт, пока ничего не останется?": "How Long Until Nothing Remains?",
};

const UI_TRANSLATIONS: Record<string, string> = {
  "Startup gate": "启动检查",
  "Checking system facts": "正在检查系统事实",
  "Recovery shell": "恢复模式",
  "Normal startup is blocked": "正常启动已阻止",
  "Diagnostic status": "诊断状态",
  Reason: "原因",
  "Supported schema": "支持的数据库结构版本",
  "Found schema": "检测到的数据库结构版本",
  "Recovery diagnostics": "恢复诊断",
  "Preview export": "预览导出",
  "Exporting…": "正在导出…",
  "Create diagnostic export": "创建诊断导出",
  "Setup shell · Workspace": "初始化 · 工作区",
  "Connect your workspace": "连接工作区",
  "Active Vault": "当前 Vault",
  "Problem Notes Root": "题目笔记目录",
  "Knowledge Root": "知识库目录",
  "Validating workspace…": "正在验证工作区…",
  "Save and enter ACM-OS": "保存并进入 ACM-OS",
  "Skip to content": "跳转到主要内容",
  Today: "今日计划",
  "Set today's budget": "设置今日预算",
  Minutes: "分钟",
  "Create Today plan": "创建今日计划",
  Contests: "比赛",
  Knowledge: "知识库",
  Settings: "设置",
  Tools: "工具",
  Primary: "主导航",
  "Review focus": "复习专注模式",
  "Contest detail": "比赛详情",
  "Problem statement": "题面",
  "Isolated review workspace": "独立复习空间",
  "Return to Today": "返回今日计划",
  "Review Attempt is unavailable": "复习记录不可用",
  "Loading isolated statement…": "正在加载独立题面…",
  "Cold-start attempt": "首次冷启动复习",
  "Open original OJ": "打开原 OJ",
  "Open controlled help": "打开受控帮助",
  "Statement snapshot": "题面快照",
  "Finish this Review": "完成本次复习",
  "Complete from facts": "根据事实完成复习",
  "Void mistaken Attempt": "作废误开的复习",
  "Controlled help": "受控帮助",
  Close: "关闭",
  Cancel: "取消",
  Confirm: "确认",
  Reveal: "查看",
  Unavailable: "不可用",
  "Open again": "再次打开",
  "Review history": "复习历史",
  "Load Review history": "加载复习历史",
  "No Review Attempts yet.": "还没有复习记录。",
  "Start Review": "开始复习",
  "Continue Review": "继续复习",
  "Start early check": "开始提前检查",
  Workspace: "工作区",
  "System Facts backup": "系统事实备份",
  "Create backup": "创建备份",
  "Preview manual backup": "预览手动备份",
  "Inspect backup inventory": "查看备份清单",
  "Creating backup…": "正在创建备份…",
  "No published backups found.": "没有已发布的备份。",
  "Markdown authority": "Markdown 权威来源",
  "Knowledge index": "知识库索引",
  "Re-index": "重新索引",
  "Re-indexing…": "正在重新索引…",
  Search: "搜索",
  "Search name or path": "搜索名称或路径",
  "Problem index": "题目索引",
  "Contest shelf": "比赛列表",
  Import: "导入",
  Refresh: "刷新",
  Loading: "正在加载…",
  "Problem is unavailable": "题目不可用",
  "Loading problem": "正在加载题目",
  "Open original problem": "打开原题",
  "Create my note": "创建我的笔记",
  "Creating note…": "正在创建笔记…",
  "Learning lifecycle": "学习生命周期",
  "Current status:": "当前状态：",
  "Next Review due:": "下次复习日期：",
  "Updating…": "正在更新…",
  "Prerequisite knowledge suggestions": "前置知识建议",
  "Personal note actions": "个人笔记操作",
  "Delete my personal note…": "删除我的个人笔记…",
  "Delete personal note": "删除个人笔记",
  "My note": "我的笔记",
  "Known sections": "已识别章节",
  "Solution routes": "解题路线",
  "Statement capture is pending": "题面快照尚未完成",
  "Preparing the local statement…": "正在准备本地题面…",
  "Review Attempt": "复习记录",
  "Review history is temporarily unavailable; history was not changed.": "复习历史暂时不可用，历史记录没有改变。",
  "Facts, not a self-selected grade": "依据事实，而不是自选评分",
  "Submission facts": "提交事实",
  "First submission result": "首次提交结果",
  "Final result": "最终结果",
  "Total submissions": "提交次数",
  Independence: "独立性",
  Debug: "调试",
  "Unrecorded external help": "未记录的外部帮助",
  "Failure reasons": "失败原因",
  "No idea": "没有思路",
  "Implementation error": "实现错误",
  "Boundary error": "边界错误",
  "Complexity judgement error": "复杂度判断错误",
  Other: "其他",
  Accepted: "通过",
  "Wrong answer": "答案错误",
  "Time limit exceeded": "超出时间限制",
  "Memory limit exceeded": "超出内存限制",
  "Runtime error": "运行时错误",
  "Compilation error": "编译错误",
  "Evidence before reveal": "查看提示前的证据",
  "Opening this drawer records a help request.": "打开此面板会记录一次帮助请求。",
  "Unknown route": "未知路径",
  "Page not found": "页面不存在",
  "No Markdown files are currently available for manual rebinding.": "当前没有可用于手动重新绑定的 Markdown 文件。",
  "Validating the local database and workspace configuration…": "正在验证本地数据库和工作区配置…",
  "No automatic repair or destructive action is performed in B0.4.": "此处不会执行自动修复或破坏性操作。",
  "Old notes, hints, solutions, Contest history, and Review history are not loaded into this Focus view.": "本专注视图不会加载旧笔记、提示、题解、比赛历史或复习历史。",
  "The system derives Mastered, Partial, or Not passed from these facts and recorded help.": "系统会根据这些事实和已记录的帮助，推导出已掌握、部分掌握或未通过。",
  "First result detail": "首次结果详情",
  "Final result detail": "最终结果详情",
  "Final result was AC": "最终结果为 AC",
  "Idea was independent": "思路独立完成",
  "Implementation was independent": "实现独立完成",
  "No debug needed": "无需调试",
  "Debugged independently": "独立完成调试",
  "Used problem-solving help to debug": "借助解题帮助完成调试",
  None: "无",
  "Problem-solving hint": "解题提示",
  "Full solution": "完整题解",
  "Select at least one when the derived result may be Partial or Not passed.": "当推导结果可能为部分掌握或未通过时，至少选择一个原因。",
  "Usage recorded at": "使用记录时间：",
  "Opening this drawer records nothing. A successful Reveal creates an irreversible usage event before content appears.": "打开此面板不会记录任何内容。成功查看提示后，会在内容显示前创建不可撤销的使用事件。",
  "Checking current Markdown…": "正在检查当前 Markdown…",
  "Reveal Level": "查看提示等级",
  "Confirm and reveal": "确认并查看",
  "Changing the Active Vault requires a future preview-and-confirm flow.": "修改当前 Vault 需要经过预览和确认流程。",
  "Schema": "结构版本",
  "Retention preview": "保留策略预览",
  "Loading today plan…": "正在加载今日计划…",
  "Set today's budget before creating the plan.": "创建计划前请先设置今日预算。",
  Date: "日期",
  "Today override": "今日覆盖时长",
  "No live Today snapshot exists yet.": "当前还没有今日计划快照。",
  "No linked problems found.": "没有找到关联题目。",
  "No live contests found.": "没有找到比赛。",
  Retry: "重试",
  "Copy path": "复制路径",
  "This will delete the bound Markdown, downgrade the Problem to Lightweight, exit its current learning lifecycle, and cancel its active Review schedule.": "这会删除已绑定的 Markdown，将题目降级为轻量题目，退出当前学习生命周期，并取消现有复习计划。",
  "Contest history, completed Review history, and historical highest evidence will be preserved.": "比赛历史、已完成的复习历史和历史最高证据都会保留。",
  "Vault is unavailable": "Vault 不可用",
  "Note location needs attention": "笔记位置需要处理",
  "The original path is missing and no unique relocation was found. The Personal Problem was not deleted or downgraded.": "原始路径缺失，且没有找到唯一的新位置。个人题目没有被删除或降级。",
  "Confirm that this Markdown was deleted?": "确认这份 Markdown 已被删除？",
  "The note binding was restored to its current location.": "笔记绑定已恢复到当前位置。",
  "DAILY EXECUTION": "每日执行",
  "Daily execution": "每日执行",
  "No weekly default is set for this weekday. Enter any non-negative whole number of minutes; tasks still use complete 30 or 60 minute planning blocks.": "今天没有设置每周默认时长。请输入不小于 0 的整数分钟数；任务仍会按完整的 30 或 60 分钟时间块规划。",
  "Preview replan": "预览重新规划",
  "Carry-in": "结转",
  "In progress": "进行中",
  "Review": "复习",
  "Not started": "未开始",
  "First cold-start Review": "首次冷启动复习",
  "Continue learning": "继续学习",
  "Long-term Review": "长期复习",
  Relearn: "重新学习",
  Upsolve: "补题",
  Study: "学习",
  Manual: "手动加入",
  "Apply this replan?": "应用这次重新规划？",
  "Apply replan": "应用重新规划",
  "Loading today plan...": "正在加载今日计划…",
  "Plan unavailable": "计划不可用",
  "No tasks fit this budget": "没有任务适合当前预算",
  "Extra suggestions": "额外建议",
  "Add to Today": "加入今日计划",
  "Direction found, key property blocked": "已经找到方向，但关键性质卡住",
  "Formula or derivation blocked": "公式或推导卡住",
  "Algorithm known, could not implement": "知道算法，但无法完成实现",
  "Open original contest": "打开原比赛",
  "Contest management": "比赛管理",
  "Restore Contest": "恢复比赛",
  "Archive Contest": "归档比赛",
  "Preview delete": "预览删除",
  "Delete Contest": "删除比赛",
  Problems: "题目",
  "Correction history": "纠错历史",
  "Post-Contest AI Analysis": "赛后 AI 分析",
  "Paste the fixed external AI template. Preview never saves; Save/Replace stores raw text and parsed sections only.": "粘贴固定的外部 AI 模板。预览不会保存；保存或替换时只存储原始文本和解析后的区块。",
  "Raw text": "原始文本",
  "Parse preview": "解析预览",
  "Save analysis": "保存分析",
  "Replace analysis": "替换分析",
  "No saved analysis.": "暂无已保存的分析。",
  "Contest is unavailable": "比赛不可用",
  "Loading contest": "正在加载比赛",
};

function translateText(text: string): string {
  const trimmed = text.trim();
  if (/[\u0400-\u04ff]/.test(trimmed)) {
    const indexedTitle = trimmed.match(/^([A-Z]\d*)\.\s+/i);
    if (indexedTitle) {
      const title = trimmed.slice(indexedTitle[0].length);
      return `${indexedTitle[1].toUpperCase()}. ${RUSSIAN_PROBLEM_TITLES[title] ?? `Problem ${indexedTitle[1].toUpperCase()}`}`;
    }
    return RUSSIAN_PROBLEM_TITLES[trimmed] ?? "Problem";
  }
  return UI_TRANSLATIONS[trimmed] ?? text;
}

function translateTree(root: Node) {
  if (root.nodeType === 1 && (root as Element).closest("[data-i18n-skip]")) return;
  const walker = document.createTreeWalker(root, NodeFilter.SHOW_TEXT);
  const nodes: Text[] = [];
  let current: Node | null;
  while ((current = walker.nextNode())) nodes.push(current as Text);
  for (const node of nodes) {
    const next = translateText(node.nodeValue ?? "");
    if (next !== node.nodeValue) node.nodeValue = next;
  }
  if (root.nodeType === 1) {
    const element = root as HTMLElement;
    for (const attribute of ["aria-label", "aria-description", "title", "placeholder"]) {
      const value = element.getAttribute(attribute);
      if (value) element.setAttribute(attribute, translateText(value));
    }
  }
}

export function installChineseUiTranslation() {
  translateTree(document.body);
  if (typeof MutationObserver === "undefined") return () => undefined;
  const observer = new MutationObserver((mutations) => {
    for (const mutation of mutations) {
      for (const node of mutation.addedNodes) translateTree(node);
      if (mutation.type === "characterData" && mutation.target.parentNode) translateTree(mutation.target.parentNode);
    }
  });
  observer.observe(document.body, { subtree: true, childList: true, characterData: true });
  return () => observer.disconnect();
}

const QUICK_TRANSLATIONS: Array<[RegExp, string]> = [
  [/Rain is falling outside the Fairy Warehouse, so Chtholly, Nephren, and Ithea spend the afternoon playing a game in the common room\.?/gi, "仙灵仓库外正在下雨，因此 Chtholly、Nephren 和 Ithea 在公共房间里玩了一下午的游戏。"],
  [/Ithea writes three non-negative integers/gi, "Ithea 写下三个非负整数"],
  [/may perform the following operation/gi, "可以执行以下操作"],
  [/an arbitrary number of times \(possibly zero\)/gi, "任意次数（也可以是零次）"],
  [/Choose one of the three current integers and replace it with the sum of the other two current integers\.?/gi, "从当前三个整数中选择一个，将它替换为另外两个整数之和。"],
  [/The other two integers remain unchanged\.?/gi, "另外两个整数保持不变。"],
  [/Nephren wants to know the minimum range of the three integers that Chtholly can obtain\.?/gi, "Nephren 想知道 Chtholly 能得到的三个整数的最小极差。"],
  [/Help her find it!?/gi, "请帮助她求出这个结果！"],
  [/The range of a non-empty finite collection of numbers is defined as its maximum value minus its minimum value\.?/gi, "非空有限数集的极差定义为最大值减最小值。"],
  [/Each test contains multiple test cases\.?/gi, "每个测试包含多个测试用例。"],
  [/The first line contains the number of test cases/gi, "第一行包含测试用例数量"],
  [/The description of the test cases follows\.?/gi, "下面给出测试用例的描述。"],
  [/The only line of each test case contains/gi, "每个测试用例的唯一一行包含"],
  [/\\text\{([^}]*)\}/gi, "$1"],
  [/\\(?:max|min)\b/gi, ""],
  [/\bInput\b/gi, "输入"], [/\bOutput\b/gi, "输出"], [/\bExamples?\b/gi, "示例"],
  [/\bConstraints?\b/gi, "限制条件"], [/\bYou are given\b/gi, "给定"], [/\bGiven\b/gi, "给定"],
  [/\bFind\b/gi, "求"], [/\bDetermine\b/gi, "确定"], [/\bFor each\b/gi, "对于每个"],
  [/\bPrint\b/gi, "输出"], [/\bThe first line\b/gi, "第一行"], [/\bThe next line\b/gi, "下一行"],
  [/\barray\b/gi, "数组"], [/\bnumber of\b/gi, "数量"], [/\bminimum\b/gi, "最小"],
  [/\bmaximum\b/gi, "最大"], [/\bpositive integer\b/gi, "正整数"],
];

export function buildChineseQuickView(html: string): string {
  const container = document.createElement("div");
  container.innerHTML = html;
  const blocks = Array.from(container.querySelectorAll("h1,h2,h3,h4,p,li,th,td,pre"))
    .map((node) => (node.textContent ?? "").replace(/\s+/g, " ").trim())
    .filter(Boolean)
    .filter((text) => !/^\d+(?:\s+\d+){3,}$/.test(text));
  const selected: string[] = [];
  for (const block of blocks) {
    if (/^(examples?|样例|input|output|输入|输出)\b/i.test(block) && selected.length > 2) break;
    if (/^(time limit|memory limit|题目限制)/i.test(block)) continue;
    selected.push(block);
    if (selected.length >= 10) break;
  }
  const source = (selected.length ? selected.join("\n") : container.textContent ?? "").trim();
  if (!source) return "题面没有可提取的文字。";
  const normalized = source
    .replace(/\\\(|\\\)|\\\[|\\\]/g, "")
    .replace(/\\left|\\right/g, "")
    .replace(/\\text\{([^{}]*)\}/g, "$1")
    .replace(/\\(?:max|min)\b/g, (value) => value.slice(1))
    .replace(/\^\{([^{}]*)\}/g, "^$1")
    .replace(/_\{([^{}]*)\}/g, "_$1")
    .replace(/\\([{}_*])/g, "$1")
    .replace(/\s+/g, " ")
    .trim();
  const translated = QUICK_TRANSLATIONS.reduce((value, [pattern, replacement]) => value.replace(pattern, replacement), normalized);
  const paragraphs = translated.split("\n").map((line) => line.trim()).filter(Boolean);
  return paragraphs.map((line, index) => `${index === 0 ? "题意：" : ""}${line}`).join("\n\n").slice(0, 1400) + (translated.length > 1400 ? "…" : "");
}

export function displayProblemTitle(_index: string, title: string): string {
  return title;
}
