import { auth } from '$lib/server/auth';
import { fetchHackClubRealName as fetchRealNameFromToken } from '$lib/server/hackclub-profile';

export async function fetchHackClubRealName(userId: string) {
	const { accessToken } = await auth.api.getAccessToken({
		body: { providerId: 'hackclub', userId }
	});
	return fetchRealNameFromToken(accessToken);
}
