import { describe, expect, it } from 'vitest';
import { inferGithubUsername } from './profile';

describe('GitHub username inference', () => {
	it('uses the owner from GitHub repository URLs', () => {
		expect(inferGithubUsername('https://github.com/hackclub/hackxpansion')).toBe('hackclub');
		expect(inferGithubUsername('https://www.github.com/maker/project.git')).toBe('maker');
	});

	it('ignores malformed, non-GitHub, and profile-only URLs', () => {
		expect(inferGithubUsername('https://gitlab.com/maker/project')).toBeNull();
		expect(inferGithubUsername('https://github.com/maker')).toBeNull();
		expect(inferGithubUsername('not a URL')).toBeNull();
	});
});
