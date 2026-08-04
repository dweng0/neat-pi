import rss from '@astrojs/rss';
import { getCollection } from 'astro:content';

export async function GET(context) {
  const episodes = (await getCollection('episodes'))
    .filter((e) => e.data.status !== 'draft')
    .sort((a, b) => a.data.episode - b.data.episode);

  return rss({
    title: 'Neato D10 Brain Transplant',
    description:
      'Gutting a bricked Neato D10 and rebuilding it as an open, locally-controlled ROS 2 robot.',
    site: context.site,
    items: episodes.map((e) => ({
      title: `Ep. ${e.data.episode}: ${e.data.title}`,
      description: e.data.teaser ?? '',
      pubDate: e.data.pubDate,
      link: `/episodes/${e.id}/`,
    })),
  });
}
