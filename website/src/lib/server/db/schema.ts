import { sql } from 'drizzle-orm';
import {
	pgTable,
	integer,
	text,
	uuid,
	timestamp,
	pgEnum,
	jsonb,
	uniqueIndex,
	index,
	check
} from 'drizzle-orm/pg-core';
import { user } from './auth.schema';
import {
	projectStatusValues,
	projectTierValues,
	projectTypeValues,
	reviewEventTypeValues
} from '$lib/projects/domain';
import {
	type FraudReview,
	type MinutesBreakdown,
	type OutboundBody,
	type OutboundCollaborator,
	type ReviewField,
	type Reviewer
} from '../ari/outbound';

export const projectStatus = pgEnum('project_status', projectStatusValues);

export const projectType = pgEnum('project_type', projectTypeValues);

export const projectTier = pgEnum('project_tier', projectTierValues);

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
		tier: projectTier('tier'),
		md1: integer('md1'),
		md2: integer('md2'),
		activeAriExternalId: text('active_ari_external_id'),
		userId: text('user_id')
			.notNull()
			.references(() => user.id, { onDelete: 'cascade' }),
		hackatime_projects: text('hackatime_projects').array()
	},
	(table) => [
		index('project_user_id_idx').on(table.userId),
		uniqueIndex('project_active_ari_external_id_uniq').on(table.activeAriExternalId),
		check(
			'project_resistor_assignment',
			sql`(${table.type} = 'app' AND ${table.md1} IS NULL AND ${table.md2} IS NULL) OR (${table.type} = 'card' AND ${table.md1} IS NOT NULL AND ${table.md2} IS NOT NULL)`
		),
		uniqueIndex('project_card_md_pair_uniq')
			.on(table.md1, table.md2)
			.where(sql`type = 'card' AND md1 IS NOT NULL AND md2 IS NOT NULL`)
	]
);

export const journal = pgTable(
	'journal',
	{
		id: uuid('id')
			.primaryKey()
			.default(sql`uuidv7()`),
		createdAt: timestamp('created_at').defaultNow().notNull(),
		updatedAt: timestamp('updated_at')
			.$onUpdate(() => /* @__PURE__ */ new Date())
			.notNull(),
		durationInMinutes: integer('duration_in_minutes').notNull(),
		text: text('text').notNull(),
		projectId: uuid('project_id')
			.notNull()
			.references(() => project.id, { onDelete: 'cascade' })
	},
	(table) => [
		index('journal_project_id_idx').on(table.projectId),
		check('journal_duration_in_minutes_range', sql`${table.durationInMinutes} BETWEEN 1 AND 10080`)
	]
);

export const reviewEvent = pgEnum('review_event', reviewEventTypeValues);

export const review = pgTable(
	'review',
	{
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
	},
	(table) => [index('review_project_id_idx').on(table.projectId)]
);

export * from './auth.schema';
