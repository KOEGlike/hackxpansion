export type UserSubmissionProfile = {
	githubUsername: string | null;
	addressLine1: string | null;
	addressLine2: string | null;
	addressCity: string | null;
	addressRegion: string | null;
	addressPostalCode: string | null;
	addressCountry: string | null;
};

export function inferGithubUsername(repoUrl: string | null | undefined): string | null {
	if (!repoUrl) return null;

	try {
		const url = new URL(repoUrl);
		if (
			url.hostname.toLowerCase() !== 'github.com' &&
			url.hostname.toLowerCase() !== 'www.github.com'
		) {
			return null;
		}

		const [owner, repository] = url.pathname.split('/').filter(Boolean);
		return owner && repository ? owner : null;
	} catch {
		return null;
	}
}
