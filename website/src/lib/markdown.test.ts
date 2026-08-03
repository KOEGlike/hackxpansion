import { describe, expect, it } from 'vitest';
import { renderMarkdown } from './markdown';

describe('Markdown rendering', () => {
	it('syntax highlights language-tagged code fences', () => {
		const html = renderMarkdown('```rust\nfn main() {}\n```');

		expect(html).toContain('class="hljs language-rust"');
		expect(html).toContain('<span class="hljs-keyword">fn</span>');
		expect(html).toContain('<span class="hljs-title function_">main</span>');
	});

	it('falls back to escaped plain code for unknown languages', () => {
		const html = renderMarkdown('```not-a-language\n<unsafe>\n```');

		expect(html).toContain('&lt;unsafe&gt;');
		expect(html).not.toContain('class="hljs');
	});

	it('sanitizes rendered HTML', () => {
		expect(renderMarkdown('<script>alert(1)</script>')).not.toContain('<script>');
	});
});
