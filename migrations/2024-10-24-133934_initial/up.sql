-- Your SQL goes here
CREATE TABLE `payout`(
	`id` INTEGER NOT NULL PRIMARY KEY,
	`number` INTEGER NOT NULL,
	`amount` DOUBLE NOT NULL,
	`currency` VARCHAR NOT NULL,
	`status` VARCHAR NOT NULL,
	`account` VARCHAR,
	`invoice_url` VARCHAR,
	`account_id` INTEGER NOT NULL,
	`created_at` TIMESTAMP NOT NULL,
	`updated_at` TIMESTAMP NOT NULL,
	FOREIGN KEY (`account_id`) REFERENCES `account`(`id`)
);

CREATE TABLE `account`(
	`id` INTEGER NOT NULL PRIMARY KEY,
	`email` VARCHAR NOT NULL,
	`password` BINARY NOT NULL,
	`recovery` BINARY,
	`commission` DOUBLE NOT NULL,
	`full_name` VARCHAR,
	`address` VARCHAR,
	`country` VARCHAR,
	`payout_method` VARCHAR,
	`payout_instructions` TEXT,
	`notify_balance` BOOL NOT NULL DEFAULT '1',
	`created_at` TIMESTAMP NOT NULL,
	`updated_at` TIMESTAMP NOT NULL
);

CREATE TABLE `tracker`(
	`id` VARCHAR NOT NULL PRIMARY KEY,
	`label` VARCHAR NOT NULL,
	`statistics_signups` INTEGER NOT NULL DEFAULT 0,
	`account_id` INTEGER NOT NULL,
	`created_at` TIMESTAMP NOT NULL,
	`updated_at` TIMESTAMP NOT NULL,
	FOREIGN KEY (`account_id`) REFERENCES `account`(`id`)
);

CREATE TABLE `balance`(
	`id` INTEGER NOT NULL PRIMARY KEY,
	`amount` DOUBLE NOT NULL DEFAULT 0,
	`currency` VARCHAR NOT NULL,
	`released` BOOL NOT NULL DEFAULT 0,
	`trace` TEXT,
	`account_id` INTEGER NOT NULL,
	`tracker_id` VARCHAR,
	`created_at` TIMESTAMP NOT NULL,
	`updated_at` TIMESTAMP NOT NULL,
	FOREIGN KEY (`account_id`) REFERENCES `account`(`id`),
	FOREIGN KEY (`tracker_id`) REFERENCES `tracker`(`id`)
);

