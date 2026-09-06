// Route-local copy keeps the shared locale catalogs within their existing budgets.
import { i18n } from '$lib/i18n.svelte';
const messages: Record<string, [string, string]> = {
  'research.select': ['Compare two selected excerpts', '比较两段选定资料'],
  'research.selectionNote': ['Choose a project source and exact lines for each excerpt. Two nonoverlapping ranges in one file are allowed. The native reader checks access and versions.', '为每段资料选择项目来源和准确行范围。可选择同一文件中互不重叠的两段；原生读取服务会核对访问范围和版本。'],
  'research.excerpt': ['Excerpt {number}', '资料片段 {number}'],
  'research.chooseSource': ['Choose a source', '选择来源'],
  'research.startLine': ['Excerpt {number} start line', '片段 {number} 起始行'],
  'research.endLine': ['Excerpt {number} end line', '片段 {number} 结束行'],
  'research.start': ['Start excerpt comparison demo', '开始片段比较演示'],
  'research.authorizeRead': ['Allow this local demo to read only these two selected excerpts and the selected method.', '允许本地演示仅读取这两段选定资料和选用方法。'],
  'research.readAccess': ['Read access: 3 resources — two selected excerpts and the selected method.', '读取范围：3 项资料——两段选定片段和选用方法。'],
  'research.loaded': ['Loaded research excerpts', '已读取的研究片段'],
  'research.demoNote': ['Offline deterministic demo · Up to 64 turns repeat a fixed comparison of the selected excerpts. Each turn reads the two selected excerpts and the selected method. This is scripted output, not model analysis.', '离线确定性演示 · 最多 64 轮重复选定片段的固定比较内容，每轮读取两段选定资料和选用方法；输出为预设内容，并非模型分析。'],
  'research.method': ['Selected method', '选用方法'],
  'research.beforeOmitted': ['Earlier lines omitted', '已省略前文'],
  'research.afterOmitted': ['Later lines omitted', '已省略后文'],
  'research.fromBeginning': ['From the source beginning', '从来源开头读取'],
  'research.toEnd': ['Through the source end', '已读取至来源末尾'],
  'research.candidate': ['Research note candidate', '研究笔记候选'],
  'research.unsaved': ['Review only · This candidate has not been saved to the project. Capture review is unavailable in this view.', '仅供审阅 · 候选尚未保存到项目，当前视图不提供 Capture 审阅入口。'],
  'research.captureReviewNote': ['This candidate is not saved. Edit the findings, then submit for Capture review. Confirming intake adds a pending Capture; use the existing consolidation review to save changes to research state.', '候选尚未保存。可编辑各项内容后提交 Capture 审阅；确认入库仅增加待处理 Capture，仍需通过已有归并审阅保存到研究状态。'],
  'research.editFinding': ['{group} {number}', '{group} {number}'],
  'research.reviewCapture': ['Submit for Capture review', '提交 Capture 审阅'],
  'research.methods': ['Methods', '研究方法'],
  'research.conclusions': ['Conclusions', '研究结论'],
  'research.comparison': ['Comparison', '比较'],
  'research.limitations': ['Limitations', '局限'],
  'research.dismiss': ['Reject candidate', '拒绝候选'],
  'research.unavailable': ['Research context could not be read. Refresh the session before reviewing a candidate.', '无法读取研究上下文，请刷新会话后再审阅候选。'],
  'research.actionFailed': ['The research action was not accepted. Check the selected lines and project version, then refresh before retrying.', '研究操作未被接受。请检查选定行范围和项目版本，刷新后重试。'],
  'research.candidateFailed': ['This turn did not produce a valid current candidate. Check the sources and start a new comparison if their content changed.', '本轮未产生有效的当前候选。请核对来源；若内容已变更，请重新开始比较。'],
  'research.notRestored': ['Research context and unsaved candidates are not restored after App restart. Start a new excerpt comparison to load current sources.', '应用重启后不恢复研究上下文和未保存候选。请开始新的片段比较以读取当前资料。'],
  "chat.developmentOnly": ["All Chat is available in development builds. Continue research through the connected model client below.", "All Chat 当前仅用于开发构建。请通过下方已连接的模型客户端继续研究。"],
  "chat.title": [
    "All Chat",
    "All Chat 对话"
  ],
  "chat.description": [
    "A retained session with visible turns, activity and permission decisions.",
    "在同一个会话中连续提问，查看执行活动并处理权限请求。"
  ],
  "chat.agent": [
    "Agent",
    "Agent"
  ],
  "chat.demo": [
    "Offline demo",
    "离线演示"
  ],
  "chat.later": [
    "Project tools pending",
    "等待项目工具接入"
  ],
  "chat.start": [
    "Start session",
    "开始会话"
  ],
  "chat.restart": [
    "Start a new session",
    "开始新会话"
  ],
  "chat.close": [
    "Close session",
    "关闭会话"
  ],
  "chat.memory": [
    "Development preview · Desktop history is saved privately in this project; the latest session returns after restart. New sessions keep earlier logs. Browser previews are temporary. The basic demo does not read sources; excerpt comparison reads your two selected ranges and the selected method. Results require Capture intake and consolidation review before they update research state.",
    "开发预览 · 桌面历史私密保存在项目中，重启后显示最近会话；新会话保留旧日志。浏览器预览为临时记录。普通演示不读取来源，片段比较读取两段选定范围和选用方法；结果须先经过 Capture 入库和归并审阅才能更新研究状态。"
  ],
  "chat.unavailable": [
    "The session could not be loaded. Return to the project with the active session, or retry in the desktop App.",
    "无法加载会话。请返回存在活动会话的项目，或在桌面应用中重试。"
  ],
  "chat.historyUnavailable": [
    "History could not be saved or restored. Close other App instances and retry. If the problem persists, stop the App and back up the project’s hidden .qiongli folder before inspecting damaged files or the 32-session limit. Existing files are preserved.",
    "无法保存或恢复历史。请关闭其他应用实例后重试；若仍失败，请停止应用并备份项目的隐藏 .qiongli 目录，再检查损坏文件或 32 个会话的上限。原文件会保留。"
  ],
  "chat.actionFailed": [
    "The action was not accepted. Refresh the session before trying again; the project or turn may have changed.",
    "操作未被接受。请刷新会话后重试，项目或轮次可能已发生变化。"
  ],
  "chat.retry": [
    "Refresh session",
    "刷新会话"
  ],
  "chat.interrupted": [
    "This session has stopped. Your messages remain visible here. Start a new session to continue; resuming this session is unavailable.",
    "会话已中断，消息仍保留在此。请开始新会话继续，当前不支持恢复原会话。"
  ],
  "chat.empty": [
    "Start the offline demo, then send a message to try a permission decision and a second turn.",
    "开始离线演示，发送消息后可体验权限处理和连续提问。"
  ],
  "chat.timeline": [
    "Conversation timeline",
    "对话时间线"
  ],
  "chat.you": [
    "You",
    "你"
  ],
  "chat.turn": [
    "Turn {number}",
    "第 {number} 轮"
  ],
  "chat.primary": [
    "Coordinator",
    "协调 Agent"
  ],
  "chat.contextUsed": [
    "Context selected for this turn",
    "本轮选择的上下文"
  ],
  "chat.notRead": [
    "Selected only; not read",
    "仅记录选择，尚未读取"
  ],
  "chat.permission": [
    "Permission requested",
    "等待权限决定"
  ],
  "chat.permissionNote": [
    "This decision applies only to this request in this turn.",
    "决定仅适用于当前轮次的这一次请求。"
  ],
  "chat.dismissPermission": [
    "Dismiss request",
    "取消请求"
  ],
  "chat.context": [
    "Sources and context",
    "来源与上下文"
  ],
  "chat.manualContext": [
    "Include text I provide",
    "附加我提供的文本"
  ],
  "chat.contextText": [
    "Context text",
    "上下文文本"
  ],
  "chat.sources": [
    "Project sources",
    "项目来源"
  ],
  "chat.loadSources": [
    "Load source list",
    "加载来源列表"
  ],
  "chat.sourcesNote": [
    "Checkboxes record source labels for the basic demo. To read contents, choose two exact excerpts below and start the comparison demo.",
    "复选框用于记录普通演示的来源标签。要读取内容，请在下方明确选择两段资料并开始比较演示。"
  ],
  "chat.noSources": [
    "No source list loaded.",
    "尚未加载来源列表。"
  ],
  "chat.message": [
    "Message",
    "消息"
  ],
  "chat.placeholder": [
    "What would you like to explore?",
    "你想探讨什么？"
  ],
  "chat.sendHint": [
    "Enter adds a line. Use Send to submit.",
    "Enter 换行，点击发送提交。"
  ],
  "chat.cancel": [
    "Stop this turn",
    "停止本轮"
  ],
  "chat.send": [
    "Send",
    "发送"
  ],
  "chat.status.ready": [
    "Ready to start",
    "可开始"
  ],
  "chat.status.starting": [
    "Connecting",
    "连接中"
  ],
  "chat.status.idle": [
    "Ready for a message",
    "等待消息"
  ],
  "chat.status.active": [
    "Turn in progress",
    "本轮进行中"
  ],
  "chat.status.closing": [
    "Closing",
    "关闭中"
  ],
  "chat.status.closed": [
    "Session closed",
    "会话已关闭"
  ],
  "chat.status.interrupted": [
    "Session interrupted",
    "会话已中断"
  ],
  "chat.activity.pending": [
    "Pending",
    "等待中"
  ],
  "chat.activity.in_progress": [
    "In progress",
    "进行中"
  ],
  "chat.activity.completed": [
    "Completed",
    "已完成"
  ],
  "chat.activity.failed": [
    "Failed",
    "失败"
  ],
  "chat.choice.allow_once": [
    "Allow once",
    "允许一次"
  ],
  "chat.choice.reject_once": [
    "Deny once",
    "拒绝一次"
  ],
  "chat.choice.allow_always": [
    "Always allow (unavailable)",
    "始终允许（不可用）"
  ],
  "chat.choice.reject_always": [
    "Always deny (unavailable)",
    "始终拒绝（不可用）"
  ],
  "chat.turnStatus.running": [
    "Working…",
    "处理中…"
  ],
  "chat.turnStatus.completed": [
    "Turn finished · Session remains open",
    "本轮结束 · 会话保持开启"
  ],
  "chat.turnStatus.cancelled": [
    "Turn cancelled",
    "本轮已取消"
  ],
  "chat.turnStatus.timed_out": [
    "Turn timed out",
    "本轮超时"
  ],
  "chat.turnStatus.failed": [
    "Turn failed",
    "本轮失败"
  ],
  "chat.turnStatus.interrupted": [
    "Turn interrupted",
    "本轮中断"
  ]
};
export function chatText(key: string, values: Record<string, string | number> = {}): string {
  const message = messages[key]?.[i18n.locale === 'zh-CN' ? 1 : 0] ?? key;
  return message.replace(/\{(\w+)\}/g, (match, name: string) => String(values[name] ?? match));
}
