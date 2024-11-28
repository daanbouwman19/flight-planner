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

