import { defineCollection, z } from 'astro:content';
import { glob } from 'astro/loaders';

// NON-DESTRUCTIVE: both collections read the working files IN PLACE at the repo root.
// Nothing is moved or copied. The site is a reader of your workflow, not a replacement.

// The serial — episode files written by `finish up`, living in ../blog at the repo root.
const episodes = defineCollection({
  loader: glob({ pattern: '*.md', base: '../blog' }),
  schema: z.object({
    title: z.string(),
    episode: z.number(),
    pubDate: z.coerce.date(),
    sessionDate: z.coerce.date().optional(),
    status: z.string().default('published'),
    teaser: z.string().optional(),
    heroPhoto: z.string().optional(),
    seeAlso: z.array(z.string()).optional(),
  }),
});

// The reference docs — your live working md at the repo root. They have NO frontmatter
// (kept exactly as you edit them), so every field is optional. Title/kind are derived
// at render time from the filename + first heading (see src/lib/reference.ts).
const reference = defineCollection({
  loader: glob({ pattern: 'neato-d10-*.md', base: '..' }),
  schema: z
    .object({
      title: z.string().optional(),
      kind: z.string().optional(),
      updated: z.coerce.date().optional(),
      summary: z.string().optional(),
    })
    .passthrough(),
});

export const collections = { episodes, reference };
