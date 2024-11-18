-- Your SQL goes here
CREATE TABLE `aircraft`(
	`id` INTEGER NOT NULL PRIMARY KEY,
	`manufacturer` TEXT NOT NULL,
	`variant` TEXT NOT NULL,
	`icao_code` TEXT NOT NULL,
	`flown` INTEGER NOT NULL,
	`aircraft_range` INTEGER NOT NULL,
	`category` TEXT NOT NULL,
	`cruise_speed` INTEGER NOT NULL,
	`date_flown` TEXT
);

CREATE TABLE `history`(
	`id` INTEGER NOT NULL PRIMARY KEY,
	`departure_icao` TEXT NOT NULL,
	`arrival_icao` TEXT NOT NULL,
	`aircraft` INTEGER NOT NULL,
	`date` TEXT NOT NULL,
	FOREIGN KEY (`aircraft`) REFERENCES `aircraft`(`id`)
);

