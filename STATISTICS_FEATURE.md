# Flight Planner - Statistics Feature

## Overview
The Flight Planner now includes a comprehensive Statistics feature that provides users with insights into their flying activity.

## New Features

### Statistics View
- **Total Flights**: Shows the total number of flights logged in your history
- **Total Distance**: Displays the cumulative distance flown in nautical miles (NM)
- **Most Flown Aircraft**: Identifies which aircraft you've used most frequently
- **Most Visited Airport**: Shows which airport (departure or arrival) appears most often in your flight history

### How to Access
1. Launch the Flight Planner application
2. Click the "Statistics" button in the Actions panel
3. View your comprehensive flight statistics

### Technical Details

#### Database Changes
- Added a `distance` column to the `history` table to store flight distances in nautical miles
- Distances are automatically calculated using the Haversine formula when flights are logged
- Existing flights without distance data will show as 0 in the total distance calculation

#### Distance Calculation
- Uses the Haversine formula to calculate great-circle distances between airports
- Distances are calculated in nautical miles and stored as integers
- Calculation is performed automatically when a flight is marked as completed

#### Statistics Stability
- **Deterministic Results**: Statistics calculations now use stable sorting to ensure consistent results
- **Tie-Breaking Logic**: When multiple aircraft have equal flight counts, the one with the lower ID is selected
- **Airport Sorting**: When multiple airports have equal visit counts, they are sorted alphabetically by ICAO code
- This prevents the "jumping" behavior where statistics would change randomly between application runs

### Usage Example
After logging several flights, the Statistics view might show:
```
Total Flights: 15
Total Distance: 12,450 NM
Most Flown Aircraft: Boeing 737-800
Most Visited Airport: EHAM
```

### Migration
The feature includes a database migration that adds the distance column to existing installations. The migration is automatically applied when the application starts.

### Testing
The feature includes comprehensive tests to verify:
- Distance calculation accuracy
- Statistics calculation correctness
- Database schema updates
- UI integration

This enhancement makes the Flight Planner more engaging by providing users with meaningful insights about their virtual flying activities.
