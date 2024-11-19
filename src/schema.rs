diesel::table! {
    aircraft (id) {
        id -> Integer,
        manufacturer -> Text,
        variant -> Text,
        icao_code -> Text,
        flown -> Integer,
        aircraft_range -> Integer,
        category -> Text,
        cruise_speed -> Integer,
        date_flown -> Nullable<Text>,
        takeoff_distance -> Nullable<Integer>,
    }
}

diesel::table! {
    history (id) {
        id -> Integer,
        departure_icao -> Text,
        arrival_icao -> Text,
        aircraft -> Integer,
        date -> Text,
    }
}

diesel::table! {
    #[allow(non_snake_case)]
    #[allow(clippy::upper_case_acronyms)]
    Airports (ID) {
        ID -> Integer,
        Name -> Text,
        ICAO -> Text,
        PrimaryID -> Nullable<Integer>,
        Latitude -> Double,
        Longtitude -> Double,
        Elevation -> Integer,
        TransitionAltitude -> Nullable<Integer>,
        TransitionLevel -> Nullable<Integer>,
        SpeedLimit -> Nullable<Integer>,
        SpeedLimitAltitude -> Nullable<Integer>,
    }
}

diesel::table! {
    #[allow(non_snake_case)]
    Runways (ID) {
        ID -> Integer,
        AirportID -> Integer,
        Ident -> Text,
        TrueHeading -> Double,
        Length -> Integer,
        Width -> Integer,
        Surface -> Text,
        Latitude -> Double,
        Longtitude -> Double,
        Elevation -> Integer,
    }
}
