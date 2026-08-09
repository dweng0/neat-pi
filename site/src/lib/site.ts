// Blog-wide identity + config. Edit these in one place.
export const SITE = {
  brand: 'Housekeeper Systems',
  tagline: 'a build blog',
  description:
    'Long-form build stories, documented episode by episode as they actually happen — the wrong turns left in.',
  // Powers the nav GitHub pill.
  githubUrl: 'https://github.com/dweng0/neat-pi',
};

// Nav links, in order. `match` is used to highlight the active tab (August's solid-cyan block).
export const NAV = [
  { label: 'Home', href: '/', match: /^\/$/ },
  { label: 'Blog', href: '/blog/', match: /^\/(blog|episodes|serials)/ },
  { label: 'About', href: '/about/', match: /^\/about/ },
];
