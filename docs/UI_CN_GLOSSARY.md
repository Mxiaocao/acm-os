# UI-CN Glossary v1

## Language Boundary

- Translate application UI through semantic message keys.
- Preserve domain identifiers, enum values, IPC fields, database values, external ACM content, and user content.
- Keep the legacy runtime translator during migration; do not add new UI copy to its map.

## Frozen Terms

| Context | English | zh-CN |
| --- | --- | --- |
| navigation | Today | 今日 |
| page title | Today plan | 今日计划 |
| navigation | Contests | 比赛 |
| navigation | My problems | 我的题库 |
| navigation | Review | 复习 |
| navigation | Knowledge | 知识库 |
| navigation | Reward | 奖励 |
| navigation | Settings | 设置 |
| problem | Learning status | 学习状态 |
| problem | Mark understood | 标记为已掌握 |
| review | Scheduled Review | 计划复习 |
| review | Early Check | 提前检查 |
| review | First Cold Start | 首次复习 |
| review | Long-term Review | 长期复习 |
| review | Review History | 复习记录 |
| reward | Experience (XP) | 经验值（XP） |
| reward | Coin | 金币 |
| reward | Level | 等级 |
| reward | Custom Reward | 自定义奖励 |
| reward | Redeem | 兑换 |
| reward | Refund | 撤销兑换 |
| reward | Enable Reward Mode | 启用奖励模式 |

Internal values such as `scheduled_review`, `early_check`, and `coin` remain unchanged. Contest/problem statements, URLs, Markdown, Personal Notes, and custom names are never passed through this catalog.
