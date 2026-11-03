import { ansiToHtml, escapeAnsi } from '@casoon/pages-theme/ansi';
import type { ShowcaseExample } from '@casoon/pages-theme/showcase';

// The input page and what `barrierlab examples/page.html` printed for it.
const files = import.meta.glob<string>('../../examples/page.{html,txt}', {
  query: '?raw',
  import: 'default',
  eager: true,
});

export const examples: ShowcaseExample[] = [
  {
    slug: 'contact-page',
    title: 'A contact page with six defects',
    file: 'examples/page.html',
    tags: ['html', 'forms', 'names', 'contrast'],
    description:
      'Grey text on a light grey page, an image without alt text, a link and a button without a name, an unlabelled field.',
    input: { code: escapeAnsi(files['../../examples/page.html']) },
    output: { html: ansiToHtml(files['../../examples/page.txt']), kind: 'terminal' },
  },
];
