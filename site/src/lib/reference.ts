// The reference docs carry no frontmatter (they're your live working files), so we derive
// their display metadata here rather than adding frontmatter to your docs.

export const REFERENCE_META: Record<string, { title: string; kind: string; order: number }> = {
  'neato-d10-brain-transplant': { title: 'The Build Doc', kind: 'build-doc', order: 0 },
  'neato-d10-handoff': { title: 'Handoff', kind: 'handoff', order: 1 },
  'neato-d10-measuring-motor-current': { title: 'Measuring Motor Current', kind: 'procedure', order: 2 },
};

/** First `# heading` in the markdown body, falling back to the mapped title or the id. */
export function refTitle(id: string, body?: string): string {
  const meta = REFERENCE_META[id];
  const heading = body?.match(/^#\s+(.+)$/m)?.[1]?.trim();
  return meta?.title ?? heading ?? id;
}

export function refKind(id: string): string {
  return REFERENCE_META[id]?.kind ?? 'doc';
}

export function refOrder(id: string): number {
  return REFERENCE_META[id]?.order ?? 99;
}
