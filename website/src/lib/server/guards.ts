import { error } from '@sveltejs/kit';

export function requireUser(locals: App.Locals) {
	if (!locals.user) error(401, 'Unauthorized');
	return locals.user;
}
