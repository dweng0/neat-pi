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

## Site structure (how episodes are organised now)

The site (`site/`, live at **blog.housekeeper.systems**) is a **multi-serial blog** — it can hold several long-form build stories, each told episode by episode. Right now there's one serial: the **Neato D10 Brain Transplant**.

- An **episode** = one post in a serial. Files still live in `./blog/NN-title.md` exactly as before.
- Each episode belongs to a **serial** via the optional `serial:` frontmatter field. No field → it belongs to the Neato build (the default). So **existing and new Neato episodes need no `serial:` line.**
- Serials are registered in **`site/src/lib/serials.ts`** (slug, title, tagline, blurb, status, cover photo). To start a *new* serial: add an entry there, then tag that serial's episodes with `serial: <slug>`. Nothing else moves.
- Blog identity (name, tagline, GitHub URL) lives in **`site/src/lib/site.ts`**.
- Photos are auto-optimised (see `heroPhoto` note below) — the raw originals in `./photos` stay untouched.

You normally only touch `./blog` and the handoff. Only edit `serials.ts`/`site.ts` if the user is spinning up a new serial or changing blog-level identity.

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
   serial: neato-d10-brain-transplant   # optional — omit for the Neato build (it's the default)
   ---
   ```
   - **`heroPhoto` can be any `./photos` filename, raw and uncompressed.** The site runs it through Astro's image pipeline at build time → webp + responsive sizes. Don't pre-shrink photos; just drop the full-res file in `./photos` and name it here. It becomes the episode's hero *and* the card image in the blog grid.
   - **`serial`** ties the episode to a build story (see the Site structure note below). Leave it **off** for Neato D10 episodes — they default to `neato-d10-brain-transplant`. Only set it when writing for a *different* serial, and only after that serial is registered (see below).

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
- **Only `finish-up` writes**, and only to `./blog`, the handoff, and (when durable) the reference docs. Never touch the `site/` build output. (Exception: registering a *new serial* in `site/src/lib/serials.ts` — only when the user is starting one.)
- **Show drafts before finalising.** Both the episode and the handoff rewrite get the user's eyes before they're done.
- The site (`site/`) reads `./blog`, `./photos`, and the reference docs **in place** — writing the files updates the site content; you don't touch the build.
- **Publishing = commit + push.** Writing the files updates the local content, but the live site (blog.housekeeper.systems) only updates when the repo is pushed to `main` (the host rebuilds on push). Don't commit/push unless the user asks — offer it as the final step.
