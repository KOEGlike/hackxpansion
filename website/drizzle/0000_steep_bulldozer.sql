CREATE TYPE "public"."project_status" AS ENUM('not_submitted', 'waiting_design', 'rejected_design', 'approved_design', 'waiting_build', 'rejected_build', 'approved_build');--> statement-breakpoint
CREATE TYPE "public"."project_type" AS ENUM('card', 'app');--> statement-breakpoint
CREATE TYPE "public"."review_event" AS ENUM('approved', 'changes', 'rejected', 'reverted', 'requeued', 'fraud');--> statement-breakpoint
CREATE TABLE "app_card" (
	"app_id" uuid NOT NULL,
	"card_id" uuid NOT NULL,
	CONSTRAINT "app_card_app_id_card_id_pk" PRIMARY KEY("app_id","card_id")
);
--> statement-breakpoint
CREATE TABLE "journal" (
	"id" uuid PRIMARY KEY DEFAULT uuidv7() NOT NULL,
	"created_at" timestamp DEFAULT now() NOT NULL,
	"updated_at" timestamp NOT NULL,
	"duration_in_minutes" integer NOT NULL,
	"project_id" uuid NOT NULL
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
	"user_id" text NOT NULL,
	"hackatime_projects" text[]
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
ALTER TABLE "app_card" ADD CONSTRAINT "app_card_app_id_project_id_fk" FOREIGN KEY ("app_id") REFERENCES "public"."project"("id") ON DELETE cascade ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "app_card" ADD CONSTRAINT "app_card_card_id_project_id_fk" FOREIGN KEY ("card_id") REFERENCES "public"."project"("id") ON DELETE cascade ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "journal" ADD CONSTRAINT "journal_project_id_project_id_fk" FOREIGN KEY ("project_id") REFERENCES "public"."project"("id") ON DELETE cascade ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "project" ADD CONSTRAINT "project_user_id_user_id_fk" FOREIGN KEY ("user_id") REFERENCES "public"."user"("id") ON DELETE cascade ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "review" ADD CONSTRAINT "review_project_id_project_id_fk" FOREIGN KEY ("project_id") REFERENCES "public"."project"("id") ON DELETE set null ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "account" ADD CONSTRAINT "account_user_id_user_id_fk" FOREIGN KEY ("user_id") REFERENCES "public"."user"("id") ON DELETE cascade ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "session" ADD CONSTRAINT "session_user_id_user_id_fk" FOREIGN KEY ("user_id") REFERENCES "public"."user"("id") ON DELETE cascade ON UPDATE no action;--> statement-breakpoint
CREATE INDEX "app_card_card_id_idx" ON "app_card" USING btree ("card_id");--> statement-breakpoint
CREATE INDEX "account_userId_idx" ON "account" USING btree ("user_id");--> statement-breakpoint
CREATE INDEX "session_userId_idx" ON "session" USING btree ("user_id");--> statement-breakpoint
CREATE INDEX "verification_identifier_idx" ON "verification" USING btree ("identifier");