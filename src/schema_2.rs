// @generated automatically by Diesel CLI.
diesel::table! {
    Airports (ID) {
        ID -> Integer,
        Name -> Nullable<Text>,
        ICAO -> Nullable<Text>,
        PrimaryID -> Nullable<Integer>,
        Latitude -> Nullable<Double>,
        Longtitude -> Nullable<Double>,
        Elevation -> Nullable<Integer>,
        TransitionAltitude -> Nullable<Integer>,
        TransitionLevel -> Nullable<Integer>,
        SpeedLimit -> Nullable<Integer>,
        SpeedLimitAltitude -> Nullable<Integer>,
    }
}

diesel::table! {
    Runways (ID) {
        ID -> Nullable<Integer>,
        AirportID -> Nullable<Integer>,
        Ident -> Nullable<Text>,
        TrueHeading -> Nullable<Double>,
        Length -> Nullable<Integer>,
        Width -> Nullable<Integer>,
        Surface -> Nullable<Text>,
        Latitude -> Nullable<Double>,
        Longtitude -> Nullable<Double>,
        Elevation -> Nullable<Integer>,
    }
}
