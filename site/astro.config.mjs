// @ts-check
import casoonPages from '@casoon/pages-theme';
import { defineConfig } from 'astro/config';

// Project page: https://casoon.github.io/<repo>/ — `base` is the GitHub Pages path.
export default defineConfig({
  site: 'https://casoon.github.io/barrierlab-cli',
  base: '/barrierlab-cli/',
  integrations: [
    casoonPages({
      name: 'barrierlab-cli',
      description:
        'Accessibility audit of a web page from its source. One command, no browser, the barrierlab rule set.',
      tagline: 'Accessibility audit without a browser',
      repo: 'casoon/barrierlab-cli',
      version: '0.1.0',
      license: 'MIT',
      packages: [],
      catalog: false,
      docsGroups: {
        'getting-started': 'Getting started',
        reference: 'Reference',
      },
    }),
  ],
});
