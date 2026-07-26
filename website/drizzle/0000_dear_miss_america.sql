CREATE TYPE "public"."project_status" AS ENUM('not_submitted', 'waiting_design', 'rejected_design', 'approved_design', 'waiting_build', 'rejected_build', 'approved_build');--> statement-breakpoint
CREATE TYPE "public"."project_tier" AS ENUM('pro', 'advanced', 'basic');--> statement-breakpoint
CREATE TYPE "public"."project_type" AS ENUM('card', 'app');--> statement-breakpoint
CREATE TYPE "public"."review_event" AS ENUM('approved', 'changes', 'rejected', 'reverted', 'requeued', 'fraud');--> statement-breakpoint
CREATE TABLE "journal" (
	"id" uuid PRIMARY KEY DEFAULT uuidv7() NOT NULL,
	"created_at" timestamp DEFAULT now() NOT NULL,
	"updated_at" timestamp NOT NULL,
	"duration_in_minutes" integer NOT NULL,
	"text" text NOT NULL,
	"project_id" uuid NOT NULL,
	CONSTRAINT "journal_duration_in_minutes_range" CHECK ("journal"."duration_in_minutes" BETWEEN 1 AND 10080)
);
--> statement-breakpoint
CREATE TABLE "project" (
	"id" uuid PRIMARY KEY DEFAULT uuidv7() NOT NULL,
	"title" text NOT NULL,
	"description" text,
	"repo_url" text,
	"demo_url" text,
	"thumbnail_url" text,
	"status" "project_status" DEFAULT 'not_submitted' NOT NULL,
	"type" "project_type" DEFAULT 'card' NOT NULL,
	"tier" "project_tier",
	"md1" integer,
	"md2" integer,
	"active_ari_external_id" text,
	"user_id" text NOT NULL,
	"hackatime_projects" text[],
	CONSTRAINT "project_resistor_assignment" CHECK (("project"."type" = 'app' AND "project"."md1" IS NULL AND "project"."md2" IS NULL) OR ("project"."type" = 'card' AND "project"."md1" IS NOT NULL AND "project"."md2" IS NOT NULL))
);
--> statement-breakpoint
CREATE TABLE "review" (
	"id" uuid PRIMARY KEY DEFAULT uuidv7() NOT NULL,
	"received_at" timestamp DEFAULT now() NOT NULL,
	"event" "review_event" NOT NULL,
	"ari_id" text NOT NULL,
	"delivery_id" text NOT NULL,
	"project_id" uuid,
	"minutes_breakdown" jsonb,
	"approved_minutes" integer GENERATED ALWAYS AS (
      COALESCE((minutes_breakdown->>'hackatime')::int, 0) +
      COALESCE((minutes_breakdown->>'journals')::int, 0) +
      COALESCE((minutes_breakdown->>'lapse')::int, 0) +
      COALESCE((minutes_breakdown->>'program')::int, 0)
    ) STORED,
	"note_to_maker" text,
	"audit_note" text,
	"fields" jsonb,
	"collaborators" jsonb,
	"fraud" jsonb,
	"reviewer" jsonb,
	"raw_payload" jsonb NOT NULL,
	CONSTRAINT "review_delivery_id_unique" UNIQUE("delivery_id")
);
--> statement-breakpoint
CREATE TABLE "account" (
	"id" text PRIMARY KEY NOT NULL,
	"account_id" text NOT NULL,
	"provider_id" text NOT NULL,
	"user_id" text NOT NULL,
	"access_token" text,
	"refresh_token" text,
	"id_token" text,
	"access_token_expires_at" timestamp,
	"refresh_token_expires_at" timestamp,
	"scope" text,
	"password" text,
	"created_at" timestamp DEFAULT now() NOT NULL,
	"updated_at" timestamp NOT NULL
);
--> statement-breakpoint
CREATE TABLE "session" (
	"id" text PRIMARY KEY NOT NULL,
	"expires_at" timestamp NOT NULL,
	"token" text NOT NULL,
	"created_at" timestamp DEFAULT now() NOT NULL,
	"updated_at" timestamp NOT NULL,
	"ip_address" text,
	"user_agent" text,
	"user_id" text NOT NULL,
	CONSTRAINT "session_token_unique" UNIQUE("token")
);
--> statement-breakpoint
CREATE TABLE "user" (
	"id" text PRIMARY KEY NOT NULL,
	"name" text NOT NULL,
	"email" text NOT NULL,
	"email_verified" boolean DEFAULT false NOT NULL,
	"image" text,
	"created_at" timestamp DEFAULT now() NOT NULL,
	"updated_at" timestamp DEFAULT now() NOT NULL,
	"slack_id" text NOT NULL,
	"verification_status" text NOT NULL,
	"given_name" text,
	"ysws_eligible" boolean NOT NULL,
	"pronouns" text,
	CONSTRAINT "user_email_unique" UNIQUE("email")
);
--> statement-breakpoint
CREATE TABLE "verification" (
	"id" text PRIMARY KEY NOT NULL,
	"identifier" text NOT NULL,
	"value" text NOT NULL,
	"expires_at" timestamp NOT NULL,
	"created_at" timestamp DEFAULT now() NOT NULL,
	"updated_at" timestamp DEFAULT now() NOT NULL
);
--> statement-breakpoint
ALTER TABLE "journal" ADD CONSTRAINT "journal_project_id_project_id_fk" FOREIGN KEY ("project_id") REFERENCES "public"."project"("id") ON DELETE cascade ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "project" ADD CONSTRAINT "project_user_id_user_id_fk" FOREIGN KEY ("user_id") REFERENCES "public"."user"("id") ON DELETE cascade ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "review" ADD CONSTRAINT "review_project_id_project_id_fk" FOREIGN KEY ("project_id") REFERENCES "public"."project"("id") ON DELETE set null ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "account" ADD CONSTRAINT "account_user_id_user_id_fk" FOREIGN KEY ("user_id") REFERENCES "public"."user"("id") ON DELETE cascade ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "session" ADD CONSTRAINT "session_user_id_user_id_fk" FOREIGN KEY ("user_id") REFERENCES "public"."user"("id") ON DELETE cascade ON UPDATE no action;--> statement-breakpoint
CREATE INDEX "journal_project_id_idx" ON "journal" USING btree ("project_id");--> statement-breakpoint
CREATE INDEX "project_user_id_idx" ON "project" USING btree ("user_id");--> statement-breakpoint
CREATE UNIQUE INDEX "project_active_ari_external_id_uniq" ON "project" USING btree ("active_ari_external_id");--> statement-breakpoint
CREATE UNIQUE INDEX "project_card_md_pair_uniq" ON "project" USING btree ("md1","md2") WHERE type = 'card' AND md1 IS NOT NULL AND md2 IS NOT NULL;--> statement-breakpoint
CREATE INDEX "review_project_id_idx" ON "review" USING btree ("project_id");--> statement-breakpoint
CREATE INDEX "account_userId_idx" ON "account" USING btree ("user_id");--> statement-breakpoint
CREATE INDEX "session_userId_idx" ON "session" USING btree ("user_id");--> statement-breakpoint
CREATE INDEX "verification_identifier_idx" ON "verification" USING btree ("identifier");