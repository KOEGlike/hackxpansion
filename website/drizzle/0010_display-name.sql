ALTER TABLE "user" RENAME COLUMN "name" TO "display_name";--> statement-breakpoint
UPDATE "user" SET "display_name" = COALESCE(NULLIF("slack_id", ''), "id"), "image" = NULL, "profile_checked_at" = NULL;--> statement-breakpoint
ALTER TABLE "user" DROP COLUMN "given_name";--> statement-breakpoint
ALTER TABLE "project_submission_feedback" DROP COLUMN "maker_name";--> statement-breakpoint
ALTER TABLE "project_submission_feedback" DROP COLUMN "maker_given_name";--> statement-breakpoint
UPDATE "account" SET "id_token" = NULL WHERE "provider_id" = 'hackclub';
