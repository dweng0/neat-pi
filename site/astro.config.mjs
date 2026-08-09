// @ts-check
import { defineConfig } from 'astro/config';

// Static site. No deploy adapter wired up yet — that's a later, off-work-machine step.
// `npm run dev` for local preview; `npm run build` outputs to ./dist.
export default defineConfig({
  site: 'https://blog.housekeeper.systems', // used for absolute URLs (RSS, canonical).
  markdown: {
    shikiConfig: {
      // Dual themes: light by default, dark swapped in via [data-theme="dark"] in global.css.
      themes: { light: 'github-light', dark: 'github-dark' },
      defaultColor: false,
      wrap: true,
    },
  },
});
