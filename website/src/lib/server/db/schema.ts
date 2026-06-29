import { sql } from 'drizzle-orm';
import { pgTable, integer, text, uuid, timestamp, pgEnum, jsonb } from 'drizzle-orm/pg-core';
import { user } from './auth.schema';

export const projectStatus = pgEnum('project_status', ['not_submitted', 'submitted']);

export const project = pgTable('project', {
	id: uuid('id')
		.primaryKey()
		.default(sql`uuidv7()`),
	title: text('title').notNull(),
	description: text('description'),
	repoUrl: text('repo_url'),
	demoUrl: text('demo_url'),
	thumbnailUrl: text('thumbnail_url'),
	status: projectStatus('status').notNull().default('not_submitted'),
	userId: text('user_id')
		.notNull()
		.references(() => user.id, { onDelete: 'cascade' })
});

export const journal = pgTable('journal', {
	id: uuid('id')
		.primaryKey()
		.default(sql`uuidv7()`),
	createdAt: timestamp('created_at').defaultNow().notNull(),
	updatedAt: timestamp('updated_at')
		.$onUpdate(() => /* @__PURE__ */ new Date())
		.notNull(),
	durationInMinutes: integer('duration_in_minutes').notNull(),
	projectId: uuid('project_id')
		.notNull()
		.references(() => project.id, { onDelete: 'cascade' })
});

export const reviewEvent = pgEnum('review_event', [
	'approved',
	'changes',
	'rejected',
	'reverted',
	'requeued',
	'fraud'
]);

export type MinutesBrakedown = {
	hackatime: number;
	journals: number;
	lapse: number;
	program: number;
};

export const review = pgTable('review', {
	id: uuid('id')
		.primaryKey()
		.default(sql`uuidv7()`),
	event: reviewEvent('event').notNull(),
	ariId: text('ari_id').notNull(),
	minutesBrakedown: jsonb('minutes_breakdown').$type<MinutesBrakedown>().notNull(),
	approvedMinutes: integer('approved_minutes').generatedAlwaysAs(sql`
      COALESCE((minutes_breakdown->>'hackatime')::int, 0) +
      COALESCE((minutes_breakdown->>'journals')::int, 0) +
      COALESCE((minutes_breakdown->>'lapse')::int, 0) +
      COALESCE((minutes_breakdown->>'program')::int, 0)
    `),
	noteToMakre: text('note_to_maker').notNull()
});

export * from './auth.schema';
