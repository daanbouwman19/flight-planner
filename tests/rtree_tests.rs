use flight_planner::models::airport::{Airport, SpatialAirport};
use rstar::{AABB, RTree};
use std::sync::Arc;

#[test]
fn test_spatial_airport_rtree() {
    // 1. Create Airport instances
    let airport1 = Arc::new(Airport {
        ID: 1,
        Name: "Airport 1".to_string(),
        ICAO: "APT1".to_string(),
        Latitude: 40.7128,
        Longtitude: -74.0060,
        ..Default::default()
    });

    let airport2 = Arc::new(Airport {
        ID: 2,
        Name: "Airport 2".to_string(),
        ICAO: "APT2".to_string(),
        Latitude: 34.0522,
        Longtitude: -118.2437,
        ..Default::default()
    });

    let airport3 = Arc::new(Airport {
        ID: 3,
        Name: "Airport 3".to_string(),
        ICAO: "APT3".to_string(),
        Latitude: 41.8781,
        Longtitude: -87.6298,
        ..Default::default()
    });

    // 2. Wrap them in SpatialAirport
    let spatial_airports = vec![
        SpatialAirport {
            airport: Arc::clone(&airport1),
        },
        SpatialAirport {
            airport: Arc::clone(&airport2),
        },
        SpatialAirport {
            airport: Arc::clone(&airport3),
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
