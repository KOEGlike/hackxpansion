ALTER TABLE "project" ADD COLUMN "md1" integer;--> statement-breakpoint
ALTER TABLE "project" ADD COLUMN "md2" integer;--> statement-breakpoint
CREATE UNIQUE INDEX "project_card_md_pair_uniq" ON "project" USING btree ("md1","md2") WHERE type = 'card' AND md1 IS NOT NULL AND md2 IS NOT NULL;