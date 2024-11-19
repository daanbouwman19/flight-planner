// @generated automatically by Diesel CLI.

diesel::table! {
    aircraft (id) {
        id -> Nullable<Integer>,
        manufacturer -> Nullable<Text>,
        variant -> Nullable<Text>,
        icao_code -> Nullable<Text>,
        flown -> Nullable<Integer>,
        aircraft_range -> Nullable<Integer>,
        category -> Nullable<Text>,
        cruise_speed -> Nullable<Integer>,
        date_flown -> Nullable<Text>,
        takeoff_distance -> Nullable<Integer>,
    }
}

diesel::table! {
    history (id) {
        id -> Nullable<Integer>,
        departure_icao -> Nullable<Text>,
        arrival_icao -> Nullable<Text>,
        aircraft -> Nullable<Integer>,
        date -> Nullable<Text>,
    }
}

diesel::joinable!(history -> aircraft (aircraft));

diesel::allow_tables_to_appear_in_same_query!(
    aircraft,
    history,
);
