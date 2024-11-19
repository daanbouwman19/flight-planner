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

CREATE TABLE `airports`(
	`id` INTEGER NOT NULL PRIMARY KEY,
	`name` TEXT NOT NULL,
	`icao` TEXT NOT NULL,
	`primaryid` INTEGER,
	`latitude` DOUBLE NOT NULL,
	`longtitude` DOUBLE NOT NULL,
	`elevation` INTEGER NOT NULL,
	`transitionaltitude` INTEGER,
	`transitionlevel` INTEGER,
	`speedlimit` INTEGER,
	`speedlimitaltitude` INTEGER
);

CREATE TABLE `history`(
	`id` INTEGER NOT NULL PRIMARY KEY,
	`departure_icao` TEXT NOT NULL,
	`arrival_icao` TEXT NOT NULL,
	`aircraft` INTEGER NOT NULL,
	`date` TEXT NOT NULL
);

CREATE TABLE `runways`(
	`id` INTEGER NOT NULL PRIMARY KEY,
	`airportid` INTEGER NOT NULL,
	`ident` TEXT NOT NULL,
	`trueheading` DOUBLE NOT NULL,
	`length` INTEGER NOT NULL,
	`width` INTEGER NOT NULL,
	`surface` TEXT NOT NULL,
	`latitude` DOUBLE NOT NULL,
	`longtitude` DOUBLE NOT NULL,
	`elevation` INTEGER NOT NULL
);

