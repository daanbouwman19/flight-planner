use flight_planner::models::airport::SpatialAirport;
use rstar::{AABB, RTree};
use std::sync::Arc;

mod common;
use common::create_test_airport;

#[test]
fn test_spatial_airport_rtree() {
    // 1. Create Airport instances
    let mut airport1_data = create_test_airport(1, "Airport 1", "APT1");
    airport1_data.Latitude = 40.7128;
    airport1_data.Longtitude = -74.0060;
    let airport1 = Arc::new(airport1_data);

    let mut airport2_data = create_test_airport(2, "Airport 2", "APT2");
    airport2_data.Latitude = 34.0522;
    airport2_data.Longtitude = -118.2437;
    let airport2 = Arc::new(airport2_data);

    let mut airport3_data = create_test_airport(3, "Airport 3", "APT3");
    airport3_data.Latitude = 41.8781;
    airport3_data.Longtitude = -87.6298;
    let airport3 = Arc::new(airport3_data);

    // 2. Wrap them in SpatialAirport
    let spatial_airports = vec![
        SpatialAirport {
            airport: Arc::clone(&airport1),
            longest_runway_length: 5000,
        },
        SpatialAirport {
            airport: Arc::clone(&airport2),
            longest_runway_length: 6000,
        },
        SpatialAirport {
            airport: Arc::clone(&airport3),
            longest_runway_length: 7000,
        },
    ];

    // 3. Create an RTree
    let rtree = RTree::bulk_load(spatial_airports);

    // 4. Define a search envelope (around New York)
    let envelope = AABB::from_corners([40.0, -75.0], [42.0, -73.0]);

    // 5. Locate airports within the envelope
    let found_airports: Vec<_> = rtree.locate_in_envelope(&envelope).collect();

    assert_eq!(found_airports.len(), 1);
    assert_eq!(found_airports[0].airport.ID, 1);

    // 6. Test with an envelope that contains no airports
    let empty_envelope = AABB::from_corners([0.0, 0.0], [1.0, 1.0]);
    let no_airports_found: Vec<_> = rtree.locate_in_envelope(&empty_envelope).collect();
    assert!(no_airports_found.is_empty());
}

#[test]
fn test_empty_rtree() {
    // 7. Test with an empty R-tree
    let empty_rtree: RTree<SpatialAirport> = RTree::new();
    let envelope = AABB::from_corners([0.0, 0.0], [90.0, 180.0]);
    let found_airports: Vec<_> = empty_rtree.locate_in_envelope(&envelope).collect();
    assert!(found_airports.is_empty());
}
