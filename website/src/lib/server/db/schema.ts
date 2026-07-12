import { sql } from 'drizzle-orm';
import {
	pgTable,
	integer,
	text,
	uuid,
	timestamp,
	pgEnum,
	jsonb,
	uniqueIndex
} from 'drizzle-orm/pg-core';
import { user } from './auth.schema';
import {
	eventTypeArray,
	type FraudReview,
	type MinutesBreakdown,
	type OutboundBody,
	type OutboundCollaborator,
	type ReviewField,
	type Reviewer
} from '../ari/outbound';

export const projectStatus = pgEnum('project_status', [
	'not_submitted',
	'waiting_design',
	'rejected_design',
	'approved_design',
	'waiting_build',
	'rejected_build',
	'approved_build'
]);

export const projectType = pgEnum('project_type', ['card', 'app']);

export const project = pgTable(
	'project',
	{
		id: uuid('id')
			.primaryKey()
			.default(sql`uuidv7()`),
		title: text('title').notNull(),
		description: text('description'),
		repoUrl: text('repo_url'),
		demoUrl: text('demo_url'),
		thumbnailUrl: text('thumbnail_url'),
		status: projectStatus('status').notNull().default('not_submitted'),
		type: projectType('type').notNull().default('card'),
		md1: integer('md1').notNull(),
		md2: integer('md2').notNull(),
		userId: text('user_id')
			.notNull()
			.references(() => user.id, { onDelete: 'cascade' }),
		hackatime_projects: text('hackatime_projects').array(),
		requirements: text('requirements')
	},
	(table) => [
		uniqueIndex('project_card_md_pair_uniq')
			.on(table.md1, table.md2)
			.where(sql`type = 'card' AND md1 IS NOT NULL AND md2 IS NOT NULL`)
	]
);

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

export const reviewEvent = pgEnum('review_event', eventTypeArray);

export const review = pgTable('review', {
	id: uuid('id')
		.primaryKey()
		.default(sql`uuidv7()`),
	receivedAt: timestamp('received_at').defaultNow().notNull(),
	event: reviewEvent('event').notNull(),
	ariId: text('ari_id').notNull(),
	deliveryId: text('delivery_id').notNull().unique(),
	projectId: uuid('project_id').references(() => project.id, { onDelete: 'set null' }),
	minutesBreakdown: jsonb('minutes_breakdown').$type<MinutesBreakdown | null>(),
	approvedMinutes: integer('approved_minutes').generatedAlwaysAs(sql`
      COALESCE((minutes_breakdown->>'hackatime')::int, 0) +
      COALESCE((minutes_breakdown->>'journals')::int, 0) +
      COALESCE((minutes_breakdown->>'lapse')::int, 0) +
      COALESCE((minutes_breakdown->>'program')::int, 0)
    `),
	noteToMaker: text('note_to_maker'),
	auditNote: text('audit_note'),
	fields: jsonb('fields').$type<ReviewField[] | null>(),
	collaborators: jsonb('collaborators').$type<OutboundCollaborator[] | null>(),
	fraud: jsonb('fraud').$type<FraudReview | null>(),
	reviewer: jsonb('reviewer').$type<Reviewer | null>(),
	rawPayload: jsonb('raw_payload').$type<OutboundBody>().notNull()
});

export * from './auth.schema';
