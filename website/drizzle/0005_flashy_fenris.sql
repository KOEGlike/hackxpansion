ALTER TABLE "project" ADD COLUMN "design_approved_type" "project_type";--> statement-breakpoint
UPDATE "project"
SET "design_approved_type" = "type"
WHERE "design_currency_awarded" = true;
