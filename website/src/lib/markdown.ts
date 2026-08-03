import DOMPurify from 'isomorphic-dompurify';
import hljs from 'highlight.js/lib/core';
import bash from 'highlight.js/lib/languages/bash';
import css from 'highlight.js/lib/languages/css';
import diff from 'highlight.js/lib/languages/diff';
import ini from 'highlight.js/lib/languages/ini';
import javascript from 'highlight.js/lib/languages/javascript';
import json from 'highlight.js/lib/languages/json';
import markdownLanguage from 'highlight.js/lib/languages/markdown';
import rust from 'highlight.js/lib/languages/rust';
import sql from 'highlight.js/lib/languages/sql';
import typescript from 'highlight.js/lib/languages/typescript';
import xml from 'highlight.js/lib/languages/xml';
import yaml from 'highlight.js/lib/languages/yaml';
import { Marked, Renderer } from 'marked';

hljs.registerLanguage('bash', bash);
hljs.registerLanguage('css', css);
hljs.registerLanguage('diff', diff);
hljs.registerLanguage('ini', ini);
hljs.registerLanguage('javascript', javascript);
hljs.registerLanguage('json', json);
hljs.registerLanguage('markdown', markdownLanguage);
hljs.registerLanguage('rust', rust);
hljs.registerLanguage('sql', sql);
hljs.registerLanguage('typescript', typescript);
hljs.registerLanguage('xml', xml);
hljs.registerLanguage('yaml', yaml);
hljs.registerAliases('svelte', { languageName: 'xml' });

const renderer = new Renderer();
const renderPlainCode = renderer.code.bind(renderer);

renderer.code = (token) => {
	const language = token.lang?.trim().split(/\s+/, 1)[0]?.toLowerCase();
	if (!language || !hljs.getLanguage(language)) return renderPlainCode(token);

	const highlighted = hljs.highlight(token.text, { language, ignoreIllegals: true }).value;
	return `<pre><code class="hljs language-${language}">${highlighted}</code></pre>`;
};

const markdown = new Marked({ renderer });

export function renderMarkdown(text: string): string {
	return DOMPurify.sanitize(markdown.parse(text, { async: false }) as string);
}
