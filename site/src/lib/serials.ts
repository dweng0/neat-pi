// The blog hosts one or more *serials* — long-form build stories told episode by episode.
// Right now there's only the Neato build, but the landing page and routing are built to hold
// more. To add a serial: append an entry here, then tag its episodes with `serial: <slug>` in
// their frontmatter. Episodes with no `serial` field belong to DEFAULT_SERIAL (the Neato build),
// so nothing existing needs editing.
import type { CollectionEntry } from 'astro:content';

export type SerialStatus = 'ongoing' | 'complete' | 'paused';

export interface Serial {
  slug: string;
  title: string;
  tagline: string; // short eyebrow label
  blurb: string; // one-paragraph pitch for the card + serial header
  status: SerialStatus;
  coverPhoto?: string; // bare filename in ../photos, run through the image optimiser
}

export const DEFAULT_SERIAL = 'neato-d10-brain-transplant';

export const SERIALS: Serial[] = [
  {
    slug: 'neato-d10-brain-transplant',
    title: 'Neato D10 Brain Transplant',
    tagline: 'A build serial',
    blurb:
      "Neato went bust and the cloud went dark. So I'm gutting a bricked D10 and rebuilding it " +
      'as an open, locally-controlled ROS 2 robot — Raspberry Pi, ESP32, Home Assistant. ' +
      'Documented as I go.',
    status: 'ongoing',
    coverPhoto: 'topdownview.jpg',
  },
];

export function getSerial(slug: string): Serial | undefined {
  return SERIALS.find((s) => s.slug === slug);
}

/** The serial an episode belongs to, falling back to the Neato build for untagged files. */
export function serialOf(episode: CollectionEntry<'episodes'>): string {
  return episode.data.serial ?? DEFAULT_SERIAL;
}

export const STATUS_LABEL: Record<SerialStatus, string> = {
  ongoing: 'In progress',
  complete: 'Complete',
  paused: 'Paused',
};
