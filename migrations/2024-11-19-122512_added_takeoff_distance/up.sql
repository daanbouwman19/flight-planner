-- Your SQL goes here
ALTER TABLE aircraft ADD COLUMN takeoff_distance INTEGER;

-- Set takeoff distances for each aircraft variant
UPDATE aircraft SET takeoff_distance = 1000 WHERE variant IN ('42-600', '42-600S', '72-600', '72-600F');
UPDATE aircraft SET takeoff_distance = 540 WHERE variant = 'MB-339A';
UPDATE aircraft SET takeoff_distance = 2290 WHERE variant = 'A310';
UPDATE aircraft SET takeoff_distance = 1750 WHERE variant IN ('A319 CFM', 'A319 IAE');
UPDATE aircraft SET takeoff_distance = 2190 WHERE variant IN ('A320 CFM', 'A320 IAE');
UPDATE aircraft SET takeoff_distance = 1951 WHERE variant = 'A320N';
UPDATE aircraft SET takeoff_distance = 2210 WHERE variant IN ('A321 CFM', 'A321 IAE');
UPDATE aircraft SET takeoff_distance = 2970 WHERE variant = 'A380';
UPDATE aircraft SET takeoff_distance = 3500 WHERE variant = 'An-225 Mriya';
UPDATE aircraft SET takeoff_distance = 180 WHERE variant = 'Husky A-1C';
UPDATE aircraft SET takeoff_distance = 300 WHERE variant IN ('Pitts Special S-1S', 'Pitts Special S-2S');
UPDATE aircraft SET takeoff_distance = 3400 WHERE variant = 'Concorde';
UPDATE aircraft SET takeoff_distance = 1000 WHERE variant IN ('Baron G53', 'Bonanza G36', 'D18S Twin Beech', 'King Air 350i', 'Model 17 Staggerwing', '307 Stratoliner');
UPDATE aircraft SET takeoff_distance = 2200 WHERE variant = '737 MAX 8';
UPDATE aircraft SET takeoff_distance = 2000 WHERE variant IN ('737-600', '737-700', '737-700 BBJ', '737-700 BDFS');
UPDATE aircraft SET takeoff_distance = 2650 WHERE variant IN ('737-800', '737-800 BBJ2', '737-800 BCF', '737-800 BDSF');
UPDATE aircraft SET takeoff_distance = 2830 WHERE variant IN ('737-900', '737-900ER');
UPDATE aircraft SET takeoff_distance = 3000 WHERE variant = '747-800';
UPDATE aircraft SET takeoff_distance = 2980 WHERE variant = '777-300ER';
UPDATE aircraft SET takeoff_distance = 2800 WHERE variant = '787-10';
UPDATE aircraft SET takeoff_distance = 1550 WHERE variant IN ('CRJ550ER', 'CRJ700ER');
UPDATE aircraft SET takeoff_distance = 660 WHERE variant = '208 B Grand Caravan EX';
UPDATE aircraft SET takeoff_distance = 960 WHERE variant = 'C172 Skyhawk';
UPDATE aircraft SET takeoff_distance = 475 WHERE variant = 'Cessna 152';
UPDATE aircraft SET takeoff_distance = 1010 WHERE variant = 'Citation CJ4';
UPDATE aircraft SET takeoff_distance = 3930 WHERE variant = 'Citation Longitude';
UPDATE aircraft SET takeoff_distance = 1085 WHERE variant = 'SR22T';
UPDATE aircraft SET takeoff_distance = 170 WHERE variant IN ('NXCub', 'XCub');
UPDATE aircraft SET takeoff_distance = 2100 WHERE variant = 'TBM 930';
UPDATE aircraft SET takeoff_distance = 915 WHERE variant = 'DHC-2 Beaver';
UPDATE aircraft SET takeoff_distance = 1220 WHERE variant = 'DHC-6 Twin Otter';
UPDATE aircraft SET takeoff_distance = 1410 WHERE variant IN ('DA-40 NG', 'DA-40 TDI');
UPDATE aircraft SET takeoff_distance = 2350 WHERE variant = 'DA-62';
UPDATE aircraft SET takeoff_distance = 1240 WHERE variant = 'DV20';
UPDATE aircraft SET takeoff_distance = 1060 WHERE variant IN ('C-47', 'DC-3');
UPDATE aircraft SET takeoff_distance = 1600 WHERE variant IN ('DC-6A', 'DC-6B');
UPDATE aircraft SET takeoff_distance = 850 WHERE variant = 'Optica';
UPDATE aircraft SET takeoff_distance = 1260 WHERE variant IN ('ERJ-170LR', 'ERJ-175LR');
UPDATE aircraft SET takeoff_distance = 600 WHERE variant = '330LT';
UPDATE aircraft SET takeoff_distance = 275 WHERE variant = 'CTSl';
UPDATE aircraft SET takeoff_distance = 517 WHERE variant = 'F.VII';
UPDATE aircraft SET takeoff_distance = 1315 WHERE variant IN ('F28-1000', 'F28-2000', 'F28-3000', 'F28-4000');
UPDATE aircraft SET takeoff_distance = 200 WHERE variant = 'Trimotor';
UPDATE aircraft SET takeoff_distance = 870 WHERE variant = 'G-21A goose';
UPDATE aircraft SET takeoff_distance = 3100 WHERE variant = 'HA420';
UPDATE aircraft SET takeoff_distance = 710 WHERE variant = 'A5';
UPDATE aircraft SET takeoff_distance = 450 WHERE variant = 'VL-3';
UPDATE aircraft SET takeoff_distance = 850 WHERE variant = 'JU-52';
UPDATE aircraft SET takeoff_distance = 200 WHERE variant = 'Freedomfox';
UPDATE aircraft SET takeoff_distance = 3100 WHERE variant IN ('MD-11', 'MD-11F');


-- Correct typos in variant names
UPDATE aircraft SET variant = '737-700 BDSF' WHERE variant = '737-700 BDFS';
