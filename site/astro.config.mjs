// @ts-check
import { defineConfig } from 'astro/config';

// Static site. No deploy adapter wired up yet — that's a later, off-work-machine step.
// `npm run dev` for local preview; `npm run build` outputs to ./dist.
export default defineConfig({
  site: 'https://example.com', // placeholder; only used for absolute URLs (RSS). Change when deploying.
  markdown: {
    shikiConfig: {
      theme: 'github-dark',
      wrap: true,
    },
  },
});
