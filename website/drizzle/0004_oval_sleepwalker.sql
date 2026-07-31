CREATE TYPE "public"."shop_order_status" AS ENUM('in_queue', 'fulfilled');--> statement-breakpoint
CREATE TABLE "shop_item" (
	"id" text PRIMARY KEY NOT NULL,
	"name" text NOT NULL,
	"description" text NOT NULL,
	"price" integer NOT NULL,
	"image_url" text,
	"required_module_designs" integer DEFAULT 0 NOT NULL,
	"required_app_designs" integer DEFAULT 0 NOT NULL,
	"active" boolean DEFAULT true NOT NULL,
	"sort_order" integer DEFAULT 0 NOT NULL,
	CONSTRAINT "shop_item_price_nonnegative" CHECK ("shop_item"."price" >= 0),
	CONSTRAINT "shop_item_requirements_nonnegative" CHECK ("shop_item"."required_module_designs" >= 0 AND "shop_item"."required_app_designs" >= 0)
);
--> statement-breakpoint
CREATE TABLE "shop_order" (
	"id" uuid PRIMARY KEY DEFAULT uuidv7() NOT NULL,
	"status" "shop_order_status" DEFAULT 'in_queue' NOT NULL,
	"price_paid" integer NOT NULL,
	"notes" text,
	"fulfillment_message" text,
	"created_at" timestamp DEFAULT now() NOT NULL,
	"fulfilled_at" timestamp,
	"item_id" text NOT NULL,
	"user_id" text NOT NULL,
	"fulfilled_by_user_id" text,
	CONSTRAINT "shop_order_price_paid_nonnegative" CHECK ("shop_order"."price_paid" >= 0)
);
--> statement-breakpoint
ALTER TABLE "user" ADD COLUMN "is_admin" boolean DEFAULT false NOT NULL;--> statement-breakpoint
ALTER TABLE "shop_order" ADD CONSTRAINT "shop_order_item_id_shop_item_id_fk" FOREIGN KEY ("item_id") REFERENCES "public"."shop_item"("id") ON DELETE restrict ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "shop_order" ADD CONSTRAINT "shop_order_user_id_user_id_fk" FOREIGN KEY ("user_id") REFERENCES "public"."user"("id") ON DELETE cascade ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "shop_order" ADD CONSTRAINT "shop_order_fulfilled_by_user_id_user_id_fk" FOREIGN KEY ("fulfilled_by_user_id") REFERENCES "public"."user"("id") ON DELETE set null ON UPDATE no action;--> statement-breakpoint
CREATE INDEX "shop_order_user_id_idx" ON "shop_order" USING btree ("user_id");--> statement-breakpoint
CREATE INDEX "shop_order_status_created_at_idx" ON "shop_order" USING btree ("status","created_at");--> statement-breakpoint
INSERT INTO "shop_item" (
	"id",
	"name",
	"description",
	"price",
	"required_module_designs",
	"required_app_designs",
	"sort_order"
) VALUES (
	'hackxpansion-console',
	'HackXPansion Console',
	'The main HackXPansion prize: a console built to bring your modules and apps together.',
	8,
	4,
	1,
	0
);
