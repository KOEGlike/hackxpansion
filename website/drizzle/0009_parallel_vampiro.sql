CREATE TABLE "project_submission_feedback" (
	"id" uuid PRIMARY KEY DEFAULT uuidv7() NOT NULL,
	"created_at" timestamp DEFAULT now() NOT NULL,
	"phase" text NOT NULL,
	"nps" integer NOT NULL,
	"how_did_you_hear" text,
	"what_are_we_doing_well" text,
	"how_can_we_improve" text,
	"github_username" text,
	"address_line_1" text,
	"address_line_2" text,
	"address_city" text,
	"address_region" text,
	"address_postal_code" text,
	"address_country" text,
	"project_repo_url" text,
	"project_demo_url" text,
	"project_thumbnail_url" text,
	"project_description" text,
	"maker_name" text NOT NULL,
	"maker_given_name" text,
	"maker_email" text NOT NULL,
	"maker_slack_id" text NOT NULL,
	"ari_external_id" text NOT NULL,
	"project_id" uuid NOT NULL,
	"user_id" text NOT NULL,
	CONSTRAINT "project_submission_feedback_nps_range" CHECK ("project_submission_feedback"."nps" BETWEEN 0 AND 10),
	CONSTRAINT "project_submission_feedback_phase" CHECK ("project_submission_feedback"."phase" IN ('design', 'build'))
);
--> statement-breakpoint
ALTER TABLE "review" ADD COLUMN "airtable_record_id" text;--> statement-breakpoint
ALTER TABLE "user" ADD COLUMN "github_username" text;--> statement-breakpoint
ALTER TABLE "user" ADD COLUMN "address_line_1" text;--> statement-breakpoint
ALTER TABLE "user" ADD COLUMN "address_line_2" text;--> statement-breakpoint
ALTER TABLE "user" ADD COLUMN "address_city" text;--> statement-breakpoint
ALTER TABLE "user" ADD COLUMN "address_region" text;--> statement-breakpoint
ALTER TABLE "user" ADD COLUMN "address_postal_code" text;--> statement-breakpoint
ALTER TABLE "user" ADD COLUMN "address_country" text;--> statement-breakpoint
ALTER TABLE "project_submission_feedback" ADD CONSTRAINT "project_submission_feedback_project_id_project_id_fk" FOREIGN KEY ("project_id") REFERENCES "public"."project"("id") ON DELETE cascade ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "project_submission_feedback" ADD CONSTRAINT "project_submission_feedback_user_id_user_id_fk" FOREIGN KEY ("user_id") REFERENCES "public"."user"("id") ON DELETE cascade ON UPDATE no action;--> statement-breakpoint
CREATE UNIQUE INDEX "project_submission_feedback_ari_external_id_uniq" ON "project_submission_feedback" USING btree ("ari_external_id");--> statement-breakpoint
CREATE INDEX "project_submission_feedback_project_id_idx" ON "project_submission_feedback" USING btree ("project_id");--> statement-breakpoint
CREATE INDEX "project_submission_feedback_user_id_idx" ON "project_submission_feedback" USING btree ("user_id");