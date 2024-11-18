// @generated automatically by Diesel CLI.

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

diesel::joinable!(history -> aircraft (aircraft));

diesel::allow_tables_to_appear_in_same_query!(
    aircraft,
    history,
);
