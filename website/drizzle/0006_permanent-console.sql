INSERT INTO "shop_item" (
	"id",
	"name",
	"description",
	"price",
	"image_url",
	"required_module_designs",
	"required_app_designs",
	"active",
	"sort_order"
) VALUES (
	'hackxpansion-console',
	'HackXPansion Console',
	'The main HackXPansion prize: a console built to bring your modules and apps together.',
	8,
	'/shop/console.png',
	4,
	1,
	true,
	0
) ON CONFLICT ("id") DO UPDATE SET
	"name" = EXCLUDED."name",
	"description" = EXCLUDED."description",
	"price" = EXCLUDED."price",
	"image_url" = EXCLUDED."image_url",
	"required_module_designs" = EXCLUDED."required_module_designs",
	"required_app_designs" = EXCLUDED."required_app_designs",
	"active" = EXCLUDED."active",
	"sort_order" = EXCLUDED."sort_order";--> statement-breakpoint
ALTER TABLE "shop_item" DROP CONSTRAINT "shop_item_requirements_nonnegative";--> statement-breakpoint
ALTER TABLE "shop_item" DROP COLUMN "required_module_designs";--> statement-breakpoint
ALTER TABLE "shop_item" DROP COLUMN "required_app_designs";
