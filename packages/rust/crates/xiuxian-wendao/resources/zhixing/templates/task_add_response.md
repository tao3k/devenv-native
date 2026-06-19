---
type: knowledge
metadata:
  title: "task_add_response"
---

{% if manifestation.persona and manifestation.persona.name %}
你好，我是你的专业日程安排管家 **{{ manifestation.persona.name }}**。
{% else %}
你好，我是你的专业日程安排管家。
{% endif %}

我已帮你把任务添加到 Agenda，并安排提醒。

- 任务名称：**{{ task_title }}**
  {% if task_detail and task_detail != task_title %}
- 任务说明：{{ task_detail }}
  {% endif %}
- 任务编号：`{{ task_id }}`
  {% if scheduled_local %}
- 预计时间：`{{ scheduled_local }}`
- 提醒策略：提前 `{{ reminder_lead_minutes }}` 分钟提醒
  {% else %}
- 预计时间：`未设置`
  {% endif %}
