---
name: cook
description: >-
  Resume a Neato D10 build session from the rolling handoff. Reads
  neato-d10-handoff.md, briefs the user on current state (confirmed vs TBD) and
  the NEXT ACTIONS, then gets out of the way so work can start. Use when the user
  says "lets cook", "let's cook", "cook", "resume", or opens a fresh session on
  this project and wants to pick up where they left off.
---

# cook — resume the build session

This is the **start** half of the build loop. Its job: get the user oriented fast, then step back so they can work at the bench. The other half is the `finish-up` skill.

## What to do

1. **Read the rolling handoff** — `neato-d10-handoff.md` at the repo root. This is the baton; it holds current state, not history. Treat it as the source of truth for "where things stand right now."

2. **Brief the user, tightly.** Lead with the NEXT ACTIONS. A good brief is:
   - **One line of context** — what this session is picking up (the project, the current phase).
   - **Where things stand** — the 2–4 most load-bearing confirmed facts and the open TBDs that gate progress. Don't recite the whole doc; surface what matters *now*.
   - **NEXT ACTIONS** — reproduce the handoff's next-actions list, in order, flagging which are doable immediately vs blocked (e.g. blocked on parts arriving, or on a measurement).
   - **The single most useful thing to do first**, called out plainly.

3. **Peek deeper only if needed.** For a specific task, `neato-d10-brain-transplant.md` (the build doc) has the full component inventory, architecture, and driver plan. `neato-d10-measuring-motor-current.md` has the bench procedure. Read them when the task calls for detail — don't dump them into the brief.

4. **Then get to work.** After the brief, stop narrating and help with whatever the user actually does. This skill's job is done once they're oriented.

## Rules

- **Don't write anything** in `cook`. Reading and briefing only. Writing is `finish-up`'s job.
- **Honour the doc's own honesty conventions** when briefing: `TBD` = needs measurement, `?` = unconfirmed, everything else = confirmed from hands-on teardown. Never present a TBD as settled.
- **Never invent state.** If the handoff doesn't say it, it isn't known. Say "the handoff doesn't cover that" rather than guessing.
- Keep it short. The user wants to be at the bench, not reading a summary of a summary.

## Site note (context, not part of the brief)

The public devlog is a **multi-serial blog** (`site/`, live at blog.housekeeper.systems) — the Neato build is one serial. `cook` doesn't write to it; episodes and the handoff are `finish-up`'s job. If a session ends up writing an episode or spinning up a new serial, the structure lives in `site/src/lib/serials.ts` (serials) and `site/src/lib/site.ts` (blog identity) — but that's `finish-up` territory, not the resume brief.
