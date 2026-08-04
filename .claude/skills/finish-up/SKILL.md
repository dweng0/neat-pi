---
name: finish-up
description: >-
  Wrap up a Neato D10 build session — serialise what happened into the next blog
  episode AND rewrite the rolling handoff for next time. Writes a new
  first-person devlog episode into ./blog, then overwrites neato-d10-handoff.md
  in place with fresh state and NEXT ACTIONS. Use when the user says "finish up",
  "wrap up", "write the next episode", "serialise this", or is ending a working
  session on this project.
---

# finish-up — serialise the session and pass the baton

This is the **end** half of the build loop (the start half is `cook`). One session, two outputs:

- **The episode** — the public story. A snapshot in time. First-person devlog. Goes in `./blog`.
- **The handoff** — the private baton. Current truth. Rolling (overwritten in place). `neato-d10-handoff.md`.

Do both, in this order, and **show the user each draft for edits before finalising** — episodes and handoffs are writing they own.

---

## Step 1 — gather what happened this session

Before writing, reconstruct the session honestly from the conversation:
- What did we actually do / measure / find?
- What got **confirmed** (a real reading, a verified fact)?
- What decisions got made, and why?
- What dead-ends, surprises, or things that didn't work? (These are the best part of a devlog — keep them.)
- What's now the next thing to do?

If the session was thin (nothing really happened), say so and offer to skip the episode and just refresh the handoff. Don't manufacture an episode out of nothing.

---

## Step 2 — write the next episode → `./blog`

1. **Find the next number.** List `./blog/*.md`, read the frontmatter `episode:` values, take the highest + 1. Filename: `NN-kebab-title.md` (zero-padded, e.g. `05-first-wheel-spins.md`).

2. **Frontmatter** (match the existing episodes exactly):
   ```yaml
   ---
   title: "Short, punchy, no episode number in it"
   episode: 5
   pubDate: <today, YYYY-MM-DD>
   sessionDate: <when the bench work happened, YYYY-MM-DD>
   status: published
   teaser: "One or two sentences. A hook, not a summary. Hint the tension."
   heroPhoto: some-photo.jpg   # optional — a filename from ./photos, omit if none fits
   seeAlso: [reference/handoff]  # optional cross-links
   ---
   ```

3. **Voice — match `./blog/*.md`, especially episode 4.** Read one before writing so the tone carries. The voice is:
   - **First person, past tense, plain-spoken.** "I got the meter out and…" not "The meter was used to…".
   - **Honest about dead-ends and surprises.** The multimeter that couldn't be trusted, the blower that needed nothing — those *are* the story. Don't sand them off into a tidy success.
   - **A small reveal or turn** where you can — set up a problem, then the thing you learned.
   - **Part numbers and specs as inline `code`.** `260-0016`, `14.4 V`, `stall ≈ 14.4 V ÷ R`.
   - **End on a hook** — the next action, teased. It's a serial; leave a thread.
   - Roughly 400–800 words. A blog post, not a lab report.

4. **The one hard rule: never invent a fact.** No measurement, spec, part number, or result that didn't actually happen this session or isn't already in the reference docs. If a number is still unknown, the episode says it's unknown — that honesty is the whole brand. Carry the `TBD` / confirmed / `?` distinction through.

---

## Step 3 — rewrite the rolling handoff → `neato-d10-handoff.md`

**Overwrite in place.** It's a rolling baton, not an append log — one file, always "now." (Git is the history; the user commits when they want a snapshot. Don't create dated copies.)

Keep the existing structure of `neato-d10-handoff.md` (it already has: one-paragraph context, key confirmed facts table, architecture, driver plan, parts status, **NEXT ACTIONS**, build order, open questions, links, photos). Update it to reflect this session:
- Bump **Last updated** to today.
- Move anything **confirmed** this session out of TBD/open-questions and into the confirmed facts.
- Rewrite **NEXT ACTIONS** so the top of the list is genuinely the next thing to do.
- Update parts status, open questions, and the photo list as needed.

The test: someone (or a fresh session) could `cook` from this handoff tomorrow and know exactly what to do without re-reading the conversation.

---

## Step 4 — reference docs, if a long-lived spec changed

If this session confirmed something that belongs in the **build doc** (`neato-d10-brain-transplant.md`) — a settled architecture decision, a final part choice, a measured spec that's now permanent — fold it in and bump its "last updated". Same for the **measuring procedure** doc if the method changed. Only for durable facts; day-to-day state lives in the handoff, not the build doc.

---

## Guardrails (the whole trust model)

- **Never invent measurements or specs.** Unknown stays unknown, visibly.
- **Episode = snapshot in time. Handoff = current truth.** An old episode can be "wrong" later (that's fine, it's dated); the handoff must never be.
- **Only `finish-up` writes**, and only to `./blog`, the handoff, and (when durable) the reference docs. Never touch the `site/` build output.
- **Show drafts before finalising.** Both the episode and the handoff rewrite get the user's eyes before they're done.
- The site (`site/`) reads all of this in place — you don't need to touch it. Writing the files *is* publishing.
