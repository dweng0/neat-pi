# Plan — Neato D10 build → Cloudflare site + blog serial

**Status:** BUILT (local). Astro site + 4 episodes + both skills done. Deferred: Cloudflare deploy.
**Decisions locked:** Astro + Cloudflare Pages · first-person devlog voice ·
**non-destructive** (working docs stay at repo root, untouched) · rolling handoff.

## Done so far
- ✅ `blog/` — Episodes 1–4, first-person devlog, drawn only from existing docs.
- ✅ `site/` — Astro (static). Reads `../blog/*.md` + root `neato-d10-*.md` in place via glob
  loaders. Builds clean (8 pages), RSS feed, photos via `site/public/photos` symlink. `npm run dev`.
- ✅ `.claude/skills/cook/` — "lets cook": read handoff, brief, work.
- ✅ `.claude/skills/finish-up/` — "finish up": write next episode + rewrite rolling handoff.

## Deferred (off the work machine)
- ⏸ Cloudflare Pages deploy (wrangler / Git-connect). Nothing CF exists in the repo yet.
- ⏸ Optional: upgrade hero images from plain `<img>` to Astro `<Image>` optimization.

## 0. The loop (the whole point)

You `cd` into the repo and drive a two-trigger loop:

- **"lets cook"** → reads the latest handoff, briefs you on where things stand + NEXT ACTIONS. You work.
- **"finish up"** → serialises the session into the next blog episode **and** rewrites the handoff
  for next time.

The **handoff is the baton** you pass to yourself (private, current-state). The **episode is the
public story** (a snapshot, first-person). Same session, two outputs. The site is a *reader* of
this workflow, never a replacement — it never writes to your working docs.

---

## 1. What we're building

Two things from one repo:

1. **A published site** on Cloudflare Pages, built from the existing markdown + photos.
2. **A repeatable serialisation workflow** — a `/serialise` Claude skill that turns each
   working session into the next *episode* of an ongoing devlog, and keeps the reference
   docs current.

The key idea: **two content types, different jobs.**

| Type | Job | Voice | Source |
|---|---|---|---|
| **Reference** | Stay authoritative & current. The "wiki." | Factual (as-is today) | The 3 existing md docs |
| **Episodes** | Tell the story, chronologically. The "serial." | First-person devlog | Written from sessions |

Reference answers "what's true now." Episodes answer "what happened, and what I learned."
An episode can go stale (it's a snapshot in time); a reference doc never should.

---

## 2. Stack & why

- **Astro (static output)** — native Markdown/MDX, content collections (built for a numbered
  series), image optimization for the 21 photos, first-class RSS. No server, no DB.
- **Cloudflare Pages** — static `dist/`, deploy via `wrangler pages deploy dist` or Git-connect
  the repo (build `npm run build`, output `dist`). No Cloudflare adapter needed for static.
- **RSS feed** — makes the serial actually subscribable; this is what earns the word "serial."

Rejected: raw-md-on-a-Worker (no image pipeline, no content model) and 11ty (more plumbing to
hand-build the series). Astro gives us the series model and image handling for free.

---

## 3. Repo layout (proposed) — NON-DESTRUCTIVE

The 3 md docs and `photos/` **stay exactly where they are** at the repo root. That's your working
surface — you `cd` in and edit them by hand, and `cook`/`finish-up` operate on them there. The
site is **additive**: everything new lives under `site/` and reads the root docs *in place* via an
Astro glob loader. Nothing moves; nothing you're currently editing gets disturbed.

```
Neato/                             # ← YOUR working surface (unchanged)
  neato-d10-brain-transplant.md    #    existing — the build doc  (untouched, read in place)
  neato-d10-handoff.md             #    existing — the rolling handoff (baton)
  neato-d10-measuring-motor-current.md
  photos/                          #    existing — read in place by the site
  blog/                            # NEW — episodes live here (new files only)
    01-the-robot-that-went-dumb.md
    02-cracking-it-open.md
    ...
  PLAN.md
  .claude/
    skills/
      cook/SKILL.md                # NEW — "lets cook": read handoff, brief, work
      finish-up/SKILL.md           # NEW — "finish up": write episode + rewrite handoff
  site/                            # NEW — the Astro project (a reader, never a writer)
    astro.config.mjs
    package.json
    src/
      content.config.ts            # glob loaders: reference ← ../*.md, episodes ← ../blog/*.md
      layouts/  BaseLayout · EpisodeLayout · ReferenceLayout
      components/  SeriesNav (prev/next) · StatusBadge (TBD/confirmed) · PhotoFigure
      pages/
        index.astro                # landing: latest episode + full series list + reference links
        episodes/[...slug].astro
        reference/[...slug].astro
        rss.xml.js
      styles/
```

Why `blog/` at root (not inside `site/`): episodes are *your* content, written by `finish-up`,
same tier as the handoff — kept next to your working docs, not buried in the website's source.
The site reads them from there. Photos are read from the existing `photos/` in place.

---

## 4. Content model (frontmatter schemas)

**Episode**
```yaml
title: "The robot that went dumb"
episode: 1
pubDate: 2026-08-04       # when the post publishes
sessionDate: 2026-08-03   # when the bench work actually happened
status: published
teaser: "Neato went bust, the cloud went dark, and you can't even root it. So we gut it."
heroPhoto: board.jpg      # optional, from assets/photos
seeAlso: [reference/build-doc]   # optional cross-links to the wiki
```

**Reference**
```yaml
title: "Neato D10 → ROS 2 Robot"
kind: build-doc | handoff | procedure
updated: 2026-08-04
summary: "Living build doc. Confirmed findings, TBDs, architecture."
```

A `StatusBadge` component renders the doc's own **TBD / confirmed / `?`** convention as coloured
chips, so the "living document" honesty carries over visually.

---

## 5. The serial — episode map (from existing material)

The three docs already contain ~4 episodes of story. Proposed opening arc:

1. **The robot that went dumb** — Neato dies, Vorwerk kills the cloud, the two walls (no OS to
   replace, locked boot chain), and the call: brain transplant.
2. **Cracking it open** — the teardown, the component inventory, and the best discovery of the
   project: *the LiDAR is a Neato LDS* — the most reverse-engineered scanner in hobby robotics.
3. **Which parts live, which parts die** — salvage verdicts, the ROS 2 + Pi 4 + ESP32 architecture.
4. **The motors won't tell you their limits** — sizing drivers to *stall*, the switch to winding
   resistance, and the blower surprise (brushless, prints its own current, needs no driver).

Future episodes write themselves as the work happens: parts arrive → bench bring-up → LiDAR
packet decode → MQTT bridge → SLAM/Nav2. Each real session = one episode.

Episodes 1–4 are a **retelling** of existing docs, so no facts get invented — I only change voice.

---

## 6. The two skills (the loop)

Both are project-level Claude skills under `.claude/skills/` so they travel with the repo.

### `cook` — trigger: "lets cook" / "let's cook"
1. Reads `neato-d10-handoff.md` (the rolling handoff).
2. Briefs you: where things stand, what's confirmed vs TBD, and the NEXT ACTIONS list.
3. Optionally peeks at the build doc for detail. Then gets out of the way — you work.

### `finish-up` — trigger: "finish up" / "wrap up"
1. **Writes the next episode** — scans `blog/` for the highest `episode:` number, drafts the next
   one from the session's work (what we did, found, decided, got stuck on) in first-person devlog
   voice. Dead-ends included — that's the good part of a devlog.
2. **Rewrites the rolling handoff** — overwrites `neato-d10-handoff.md` in place: new confirmed
   findings folded in, NEXT ACTIONS refreshed, `Last updated` bumped. (You `git commit` if you
   want the previous version kept — git is the history, the file is always "now.")
3. **Folds confirmed facts into the build doc** if any long-lived spec changed.
4. **Shows you both** for edits before finalising — episodes and handoffs are writing you own.

Guardrails baked in: never invent measurements or specs; carry the TBD / confirmed / `?`
convention through; **episode = snapshot in time, handoff = current truth.** Only `finish-up`
writes, and only to the root working docs + `blog/` — never silently.

---

## 7. Photos

21 files, incl. large phone shots and `.MP.jpg` motion-photo bursts (some are near-duplicates).
Plan: curate to the ones each episode/reference actually needs, run them through Astro `<Image>`
for responsive/optimized output. The docs already list "still to photograph" — that becomes a
visible TODO on the relevant reference page.

---

## 8. Deploy (later, when we build)

- `npm run build` → static `dist/`.
- **Option A (fastest):** `wrangler pages deploy dist` from the CLI.
- **Option B (hands-off):** `git init`, push to GitHub, connect repo in Cloudflare Pages dashboard
  (build `npm run build`, output `dist`) → every push auto-deploys. Fits your "blog it on GitHub"
  goal and makes each `/serialise` + push publish an episode.
- Custom domain optional, added in the Pages dashboard.

---

## 9. Build order (once you approve)

1. Scaffold Astro in `site/`, static output, base layout + styles.
2. Content collections + schemas; move the 3 md into `reference/`, wire the StatusBadge convention.
3. Move + optimize photos; add `PhotoFigure`.
4. Episodes collection + `SeriesNav`; write Episodes 1–4 from existing material.
5. Landing page + RSS.
6. `/serialise` skill.
7. `npm run build` + a local preview to verify, then deploy.

---

## 10. Resolved

- ✅ Non-destructive — working docs + photos stay at repo root, read in place. Site is `site/`.
- ✅ Rolling handoff — `finish-up` overwrites in place; `git commit` is the history.
- ✅ Two-skill loop — `cook` / `finish-up`.

## 11. Still open for you

- **Deploy path** — CLI (`wrangler`) or Git-connected auto-deploy (§8)? Git-connected pairs best
  with the loop: `finish-up` + `git push` = episode published.
- **Domain** — custom domain now, or start on the free `*.pages.dev`?
- **Episodes 1–4** — draft all four from the existing docs, or just Episode 1 as a tone sample you
  approve before I do the rest?
