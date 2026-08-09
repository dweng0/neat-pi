// Pulls the working photos (repo-root ../photos, reached here via ../../../photos) through
// Astro's asset pipeline so they get compressed + converted to webp at build time.
// The originals are never touched — Astro writes optimised copies into the build output.
// heroPhoto in the episode frontmatter is a bare filename, so we key the map by basename.
import type { ImageMetadata } from 'astro';

const modules = import.meta.glob<{ default: ImageMetadata }>(
  '../../../photos/*.{jpg,jpeg,png,JPG,JPEG,PNG}',
  { eager: true },
);

const byName: Record<string, ImageMetadata> = {};
for (const path in modules) {
  const name = path.split('/').pop()!;
  byName[name] = modules[path].default;
}

/** Look up an optimisable image by its bare filename (as stored in heroPhoto). */
export function getPhoto(name?: string): ImageMetadata | undefined {
  return name ? byName[name] : undefined;
}
