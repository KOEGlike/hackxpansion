ALTER TABLE "project" ADD COLUMN "currency_paid_out" integer DEFAULT 0 NOT NULL;--> statement-breakpoint
ALTER TABLE "project" ADD COLUMN "design_currency_awarded" boolean DEFAULT false NOT NULL;--> statement-breakpoint
ALTER TABLE "project" ADD COLUMN "build_currency_awarded" boolean DEFAULT false NOT NULL;--> statement-breakpoint
ALTER TABLE "user" ADD COLUMN "currency" integer DEFAULT 0 NOT NULL;--> statement-breakpoint
UPDATE "project"
SET
	"currency_paid_out" = CASE "tier"
		WHEN 'pro' THEN 3
		WHEN 'advanced' THEN 2
		WHEN 'basic' THEN 1
		ELSE 0
	END,
	"design_currency_awarded" = true
WHERE EXISTS (
	SELECT 1
	FROM "review"
	WHERE
		"review"."project_id" = "project"."id"
		AND "review"."event" = 'approved'
		AND split_part("review"."raw_payload"->>'external_id', ':', 2) = 'design'
);--> statement-breakpoint
UPDATE "project"
SET
	"currency_paid_out" = "currency_paid_out" + 1,
	"build_currency_awarded" = true
WHERE EXISTS (
	SELECT 1
	FROM "review"
	WHERE
		"review"."project_id" = "project"."id"
		AND "review"."event" = 'approved'
		AND split_part("review"."raw_payload"->>'external_id', ':', 2) = 'build'
);--> statement-breakpoint
UPDATE "user"
SET "currency" = "payouts"."total"
FROM (
	SELECT "user_id", SUM("currency_paid_out")::integer AS "total"
	FROM "project"
	GROUP BY "user_id"
) AS "payouts"
WHERE "user"."id" = "payouts"."user_id";--> statement-breakpoint
ALTER TABLE "project" ADD CONSTRAINT "project_currency_paid_out_nonnegative" CHECK ("project"."currency_paid_out" >= 0);--> statement-breakpoint
ALTER TABLE "user" ADD CONSTRAINT "user_currency_nonnegative" CHECK ("user"."currency" >= 0);
