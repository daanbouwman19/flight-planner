#![cfg(feature = "gui")]
use flight_planner::models::airport::SpatialAirport;
use rstar::{AABB, RTree};

mod common;
use common::create_test_spatial_airport;

#[test]
fn test_spatial_airport_rtree() {
    // 1. Create SpatialAirport instances using the helper
    let spatial_airports = vec![
        create_test_spatial_airport(1, 40.7128, -74.0060, 5000), // NY
        create_test_spatial_airport(2, 34.0522, -118.2437, 6000), // LA
        create_test_spatial_airport(3, 41.8781, -87.6298, 7000), // Chicago
    ];

    // 2. Create an RTree
    let rtree = RTree::bulk_load(spatial_airports);

    // 3. Define a search envelope (around New York)
    let envelope = AABB::from_corners([40.0, -75.0], [42.0, -73.0]);

    // 4. Locate airports within the envelope
    let found_airports: Vec<_> = rtree.locate_in_envelope(&envelope).collect();

    assert_eq!(found_airports.len(), 1);
    assert_eq!(found_airports[0].airport.inner.ID, 1);

    // 5. Test with an envelope that contains no airports
    let empty_envelope = AABB::from_corners([0.0, 0.0], [1.0, 1.0]);
    let no_airports_found: Vec<_> = rtree.locate_in_envelope(&empty_envelope).collect();
    assert!(no_airports_found.is_empty());
}

#[test]
fn test_empty_rtree() {
    // 6. Test with an empty R-tree
    let empty_rtree: RTree<SpatialAirport> = RTree::new();
    let envelope = AABB::from_corners([0.0, 0.0], [90.0, 180.0]);
    let found_airports: Vec<_> = empty_rtree.locate_in_envelope(&envelope).collect();
    assert!(found_airports.is_empty());
}
