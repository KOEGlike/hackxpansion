ALTER TABLE "app_card" DISABLE ROW LEVEL SECURITY;--> statement-breakpoint
DROP TABLE "app_card" CASCADE;--> statement-breakpoint
ALTER TABLE "project" ALTER COLUMN "md1" SET NOT NULL;--> statement-breakpoint
ALTER TABLE "project" ALTER COLUMN "md2" SET NOT NULL;