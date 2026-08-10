import { sql } from 'drizzle-orm';
import {
	pgTable,
	integer,
	text,
	uuid,
	timestamp,
	pgEnum,
	jsonb,
	boolean,
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
} from '../../projects/domain';
import type {
	FraudReview,
	MinutesBreakdown,
	OutboundBody,
	OutboundCollaborator,
	ReviewField,
	ReviewJustification,
	Reviewer
} from '../ari/outbound';
import type { ProjectReviewPhase } from '../../projects/lifecycle';

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
		currencyPaidOut: integer('currency_paid_out').default(0).notNull(),
		designCurrencyAwarded: boolean('design_currency_awarded').default(false).notNull(),
		designApprovedType: projectType('design_approved_type'),
		buildCurrencyAwarded: boolean('build_currency_awarded').default(false).notNull(),
		md0: integer('md0'),
		md1: integer('md1'),
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
			sql`(${table.type} = 'app' AND ${table.md0} IS NULL AND ${table.md1} IS NULL) OR (${table.type} = 'card' AND ${table.md0} IS NOT NULL AND ${table.md1} IS NOT NULL)`
		),
		check('project_currency_paid_out_nonnegative', sql`${table.currencyPaidOut} >= 0`),
		uniqueIndex('project_card_md_pair_uniq')
			.on(table.md0, table.md1)
			.where(sql`type = 'card' AND md0 IS NOT NULL AND md1 IS NOT NULL`)
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

export const projectSubmissionFeedback = pgTable(
	'project_submission_feedback',
	{
		id: uuid('id')
			.primaryKey()
			.default(sql`uuidv7()`),
		createdAt: timestamp('created_at').defaultNow().notNull(),
		phase: text('phase').$type<ProjectReviewPhase>().notNull(),
		nps: integer('nps').notNull(),
		howDidYouHear: text('how_did_you_hear'),
		whatAreWeDoingWell: text('what_are_we_doing_well'),
		howCanWeImprove: text('how_can_we_improve'),
		githubUsername: text('github_username'),
		addressLine1: text('address_line_1'),
		addressLine2: text('address_line_2'),
		addressCity: text('address_city'),
		addressRegion: text('address_region'),
		addressPostalCode: text('address_postal_code'),
		addressCountry: text('address_country'),
		projectRepoUrl: text('project_repo_url'),
		projectDemoUrl: text('project_demo_url'),
		projectThumbnailUrl: text('project_thumbnail_url'),
		projectDescription: text('project_description'),
		makerName: text('maker_name').notNull(),
		makerGivenName: text('maker_given_name'),
		makerEmail: text('maker_email').notNull(),
		makerSlackId: text('maker_slack_id').notNull(),
		ariExternalId: text('ari_external_id').notNull(),
		projectId: uuid('project_id')
			.notNull()
			.references(() => project.id, { onDelete: 'cascade' }),
		userId: text('user_id')
			.notNull()
			.references(() => user.id, { onDelete: 'cascade' })
	},
	(table) => [
		uniqueIndex('project_submission_feedback_ari_external_id_uniq').on(table.ariExternalId),
		index('project_submission_feedback_project_id_idx').on(table.projectId),
		index('project_submission_feedback_user_id_idx').on(table.userId),
		check('project_submission_feedback_nps_range', sql`${table.nps} BETWEEN 0 AND 10`),
		check('project_submission_feedback_phase', sql`${table.phase} IN ('design', 'build')`)
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
		justification: jsonb('justification').$type<ReviewJustification | null>(),
		fields: jsonb('fields').$type<ReviewField[] | null>(),
		collaborators: jsonb('collaborators').$type<OutboundCollaborator[] | null>(),
		fraud: jsonb('fraud').$type<FraudReview | null>(),
		reviewer: jsonb('reviewer').$type<Reviewer | null>(),
		rawPayload: jsonb('raw_payload').$type<OutboundBody>().notNull(),
		airtableRecordId: text('airtable_record_id')
	},
	(table) => [index('review_project_id_idx').on(table.projectId)]
);

export const shopOrderStatusValues = ['in_queue', 'fulfilled'] as const;
export const shopOrderStatus = pgEnum('shop_order_status', shopOrderStatusValues);

export const shopItem = pgTable(
	'shop_item',
	{
		id: text('id').primaryKey(),
		name: text('name').notNull(),
		description: text('description').notNull(),
		price: integer('price').notNull(),
		imageUrl: text('image_url'),
		active: boolean('active').default(true).notNull(),
		sortOrder: integer('sort_order').default(0).notNull()
	},
	(table) => [check('shop_item_price_nonnegative', sql`${table.price} >= 0`)]
);

export const shopOrder = pgTable(
	'shop_order',
	{
		id: uuid('id')
			.primaryKey()
			.default(sql`uuidv7()`),
		status: shopOrderStatus('status').default('in_queue').notNull(),
		pricePaid: integer('price_paid').notNull(),
		notes: text('notes'),
		fulfillmentMessage: text('fulfillment_message'),
		createdAt: timestamp('created_at').defaultNow().notNull(),
		fulfilledAt: timestamp('fulfilled_at'),
		itemId: text('item_id')
			.notNull()
			.references(() => shopItem.id, { onDelete: 'restrict' }),
		userId: text('user_id')
			.notNull()
			.references(() => user.id, { onDelete: 'cascade' }),
		fulfilledByUserId: text('fulfilled_by_user_id').references(() => user.id, {
			onDelete: 'set null'
		})
	},
	(table) => [
		index('shop_order_user_id_idx').on(table.userId),
		index('shop_order_status_created_at_idx').on(table.status, table.createdAt),
		check('shop_order_price_paid_nonnegative', sql`${table.pricePaid} >= 0`)
	]
);

export * from './auth.schema';
