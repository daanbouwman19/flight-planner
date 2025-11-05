#!/usr/bin/env python3
"""
Analyze the real airport database to extract statistical distributions
for generating accurate mock data.

Usage: python analyze_airport_database.py [path/to/airports.db3]
"""

import sqlite3
import sys
import os
from pathlib import Path


def get_percentile(cursor, table, column, percentile, where_clause=None):
    """Get a specific percentile value from a column.
    
    Args:
        cursor: Database cursor
        table: Table name (must be 'airports' or 'runways')
        column: Column name
        percentile: Percentile value (0.0 to 1.0)
        where_clause: Optional WHERE clause (e.g., 'Length > 0')
    """
    # Validate table name to prevent SQL injection
    if table not in ['airports', 'runways']:
        raise ValueError(f"Invalid table name: {table}")
    
    where_sql = f"WHERE {where_clause}" if where_clause else ""
    
    cursor.execute(f"""
        SELECT {column} 
        FROM {table} 
        {where_sql}
        ORDER BY {column} 
        LIMIT 1 
        OFFSET (SELECT CAST(COUNT(*) * ? AS INTEGER) FROM {table} {where_sql})
    """, (percentile,))
    result = cursor.fetchone()
    return result[0] if result else None


def analyze_airports(cursor):
    """Analyze airport statistics."""
    print("=" * 80)
    print("AIRPORT DATABASE STATISTICAL ANALYSIS")
    print("=" * 80)
    
    # Total counts
    cursor.execute("SELECT COUNT(*) FROM airports")
    airport_count = cursor.fetchone()[0]
    print(f"\nTotal Airports: {airport_count:,}")
    
    # Elevation statistics
    print("\n" + "-" * 80)
    print("ELEVATION DISTRIBUTION (feet)")
    print("-" * 80)
    cursor.execute("""
        SELECT 
            MIN(Elevation) as min,
            AVG(Elevation) as mean,
            MAX(Elevation) as max
        FROM airports
    """)
    min_elev, mean_elev, max_elev = cursor.fetchone()
    
    percentiles = [0.10, 0.25, 0.50, 0.75, 0.90, 0.95, 0.99]
    print(f"Min:     {min_elev:>8.0f} ft")
    print(f"Mean:    {mean_elev:>8.0f} ft")
    for p in percentiles:
        val = get_percentile(cursor, 'airports', 'Elevation', p)
        print(f"P{int(p*100):>2}:     {val:>8.0f} ft")
    print(f"Max:     {max_elev:>8.0f} ft")
    
    # Geographic distribution
    print("\n" + "-" * 80)
    print("GEOGRAPHIC DISTRIBUTION")
    print("-" * 80)
    cursor.execute("""
        SELECT 
            MIN(Latitude) as min_lat,
            MAX(Latitude) as max_lat,
            MIN(Longtitude) as min_lon,
            MAX(Longtitude) as max_lon
        FROM airports
    """)
    min_lat, max_lat, min_lon, max_lon = cursor.fetchone()
    print(f"Latitude:  {min_lat:>8.2f}° to {max_lat:>8.2f}°")
    print(f"Longitude: {min_lon:>8.2f}° to {max_lon:>8.2f}°")


def analyze_runways(cursor):
    """Analyze runway statistics."""
    
    # Total runways
    cursor.execute("SELECT COUNT(*) FROM runways")
    runway_count = cursor.fetchone()[0]
    print(f"\nTotal Runways: {runway_count:,}")
    
    # Runways per airport
    print("\n" + "-" * 80)
    print("RUNWAYS PER AIRPORT DISTRIBUTION")
    print("-" * 80)
    cursor.execute("""
        SELECT 
            CAST(COUNT(*) AS FLOAT) / COUNT(DISTINCT AirportID) as avg_runways
        FROM runways
    """)
    avg_runways = cursor.fetchone()[0]
    print(f"Average runways per airport: {avg_runways:.2f}")
    
    # Distribution by count
    cursor.execute("""
        SELECT 
            runway_count,
            COUNT(*) as airport_count,
            CAST(COUNT(*) * 100.0 / (SELECT COUNT(DISTINCT AirportID) FROM runways) AS REAL) as percentage
        FROM (
            SELECT AirportID, COUNT(*) as runway_count 
            FROM runways 
            GROUP BY AirportID
        )
        GROUP BY runway_count
        ORDER BY runway_count
    """)
    print("\nRunway Count Distribution:")
    print(f"{'Count':<10} {'Airports':<15} {'Percentage':<10}")
    print("-" * 40)
    for count, airports, pct in cursor.fetchall():
        print(f"{count:<10} {airports:<15,} {pct:>6.2f}%")
    
    # Runway length statistics
    print("\n" + "-" * 80)
    print("RUNWAY LENGTH DISTRIBUTION (feet)")
    print("-" * 80)
    cursor.execute("""
        SELECT 
            MIN(Length) as min,
            AVG(Length) as mean,
            MAX(Length) as max
        FROM runways
        WHERE Length > 0
    """)
    min_len, mean_len, max_len = cursor.fetchone()
    
    print(f"Min:     {min_len:>8.0f} ft")
    print(f"Mean:    {mean_len:>8.0f} ft")
    percentiles = [0.10, 0.25, 0.50, 0.75, 0.90, 0.95, 0.99]
    for p in percentiles:
        val = get_percentile(cursor, 'runways', 'Length', p, 'Length > 0')
        print(f"P{int(p*100):>2}:     {val:>8.0f} ft")
    print(f"Max:     {max_len:>8.0f} ft")
    
    # Runway width statistics
    print("\n" + "-" * 80)
    print(f"RUNWAY WIDTH DISTRIBUTION (feet)")
    print("-" * 80)
    cursor.execute("""
        SELECT 
            MIN(Width) as min,
            AVG(Width) as mean,
            MAX(Width) as max
        FROM runways
        WHERE Width > 0
    """)
    min_width, mean_width, max_width = cursor.fetchone()
    
    print(f"Min:     {min_width:>8.0f} ft")
    print(f"Mean:    {mean_width:>8.0f} ft")
    for p in percentiles:
        val = get_percentile(cursor, 'runways', 'Width', p, 'Width > 0')
        print(f"P{int(p*100):>2}:     {val:>8.0f} ft")
    print(f"Max:     {max_width:>8.0f} ft")
    
    # Surface type distribution
    print("\n" + "-" * 80)
    print("RUNWAY SURFACE TYPE DISTRIBUTION")
    print("-" * 80)
    cursor.execute("""
        SELECT 
            Surface,
            COUNT(*) as count,
            CAST(COUNT(*) * 100.0 / (SELECT COUNT(*) FROM runways) AS REAL) as percentage
        FROM runways
        GROUP BY Surface
        ORDER BY count DESC
    """)
    print(f"{'Surface':<15} {'Count':<15} {'Percentage':<10}")
    print("-" * 40)
    for surface, count, pct in cursor.fetchall():
        surface_display = surface if surface else "(null)"
        print(f"{surface_display:<15} {count:<15,} {pct:>6.2f}%")


def generate_rust_code(cursor):
    """Generate Rust constants for mock data based on statistics."""
    
    print("\n" + "=" * 80)
    print("RUST CONSTANTS FOR mock_data.rs")
    print("=" * 80)
    print("// Copy these constants to the top of mock_data.rs")
    print("// They are based on statistical analysis of the real database")
    
    # Get total airport count
    cursor.execute("SELECT COUNT(*) FROM airports")
    airport_count = cursor.fetchone()[0]
    
    print(f"\n/// Default airport count based on real database size")
    print(f"pub const DEFAULT_AIRPORT_COUNT: usize = {airport_count};")
    
    # Get elevation statistics
    cursor.execute("SELECT MIN(Elevation), MAX(Elevation) FROM airports")
    min_elev, max_elev = cursor.fetchone()
    
    percentiles = {}
    for p in [0.25, 0.50, 0.75, 0.90, 0.95]:
        percentiles[p] = get_percentile(cursor, 'airports', 'Elevation', p)
    
    print(f"\n/// Elevation distribution constants (feet)")
    print(f"const ELEVATION_MIN: i32 = {int(min_elev)};")
    print(f"const ELEVATION_MAX: i32 = {int(max_elev)};")
    print(f"const ELEVATION_P25: i32 = {int(percentiles[0.25])};")
    print(f"const ELEVATION_P50: i32 = {int(percentiles[0.50])};")
    print(f"const ELEVATION_P75: i32 = {int(percentiles[0.75])};")
    print(f"const ELEVATION_P90: i32 = {int(percentiles[0.90])};")
    print(f"const ELEVATION_P95: i32 = {int(percentiles[0.95])};")
    
    # Get latitude/longitude ranges
    cursor.execute("""
        SELECT 
            MIN(Latitude) as min_lat,
            MAX(Latitude) as max_lat,
            MIN(Longtitude) as min_lon,
            MAX(Longtitude) as max_lon
        FROM airports
    """)
    min_lat, max_lat, min_lon, max_lon = cursor.fetchone()
    
    print(f"\n/// Geographic distribution constants")
    print(f"const LATITUDE_MIN: f64 = {min_lat:.2f};")
    print(f"const LATITUDE_MAX: f64 = {max_lat:.2f};")
    print(f"const LONGITUDE_MIN: f64 = {min_lon:.2f};")
    print(f"const LONGITUDE_MAX: f64 = {max_lon:.2f};")
    
    # Runway count distribution
    cursor.execute("""
        SELECT 
            runway_count,
            CAST(COUNT(*) * 100.0 / (SELECT COUNT(DISTINCT AirportID) FROM runways) AS REAL) as percentage
        FROM (
            SELECT AirportID, COUNT(*) as runway_count
            FROM runways
            GROUP BY AirportID
        )
        GROUP BY runway_count
        ORDER BY runway_count
    """)
    runway_dist = cursor.fetchall()
    
    print(f"\n/// Runway count distribution [(count, cumulative_probability)]")
    print("const RUNWAY_COUNT_DISTRIBUTION: &[(i32, f64)] = &[")
    cumulative = 0.0
    for count, pct in runway_dist:
        cumulative += pct / 100.0
        print(f"    ({count}, {cumulative:.4f}),  // {pct:.2f}%")
    print("];")
    
    # Runway length distribution
    cursor.execute("SELECT MIN(Length), MAX(Length) FROM runways WHERE Length > 0")
    min_length, max_length = cursor.fetchone()
    
    length_percentiles = {}
    for p in [0.25, 0.50, 0.75, 0.90]:
        length_percentiles[p] = get_percentile(cursor, 'runways', 'Length', p, 'Length > 0')
    
    print(f"\n/// Runway length distribution constants (feet)")
    print(f"const RUNWAY_LENGTH_MIN: i32 = {int(min_length)};")
    print(f"const RUNWAY_LENGTH_MAX: i32 = {int(max_length)};")
    print(f"const RUNWAY_LENGTH_P25: i32 = {int(length_percentiles[0.25])};")
    print(f"const RUNWAY_LENGTH_P50: i32 = {int(length_percentiles[0.50])};")
    print(f"const RUNWAY_LENGTH_P75: i32 = {int(length_percentiles[0.75])};")
    print(f"const RUNWAY_LENGTH_P90: i32 = {int(length_percentiles[0.90])};")
    
    # Runway width distribution
    cursor.execute("SELECT MIN(Width), MAX(Width) FROM runways WHERE Width > 0")
    min_width, max_width = cursor.fetchone()
    
    width_percentiles = {}
    for p in [0.25, 0.50, 0.75, 0.90]:
        width_percentiles[p] = get_percentile(cursor, 'runways', 'Width', p, 'Width > 0')
    
    print(f"\n/// Runway width distribution constants (feet)")
    print(f"const RUNWAY_WIDTH_MIN: i32 = {int(min_width)};")
    print(f"const RUNWAY_WIDTH_MAX: i32 = {int(max_width)};")
    print(f"const RUNWAY_WIDTH_P25: i32 = {int(width_percentiles[0.25])};")
    print(f"const RUNWAY_WIDTH_P50: i32 = {int(width_percentiles[0.50])};")
    print(f"const RUNWAY_WIDTH_P75: i32 = {int(width_percentiles[0.75])};")
    print(f"const RUNWAY_WIDTH_P90: i32 = {int(width_percentiles[0.90])};")
    
    # Surface type distribution
    cursor.execute("""
        SELECT 
            Surface,
            CAST(COUNT(*) * 100.0 / (SELECT COUNT(*) FROM runways) AS REAL) as percentage
        FROM runways
        GROUP BY Surface
        ORDER BY COUNT(*) DESC
        LIMIT 10
    """)
    surfaces = cursor.fetchall()
    
    print(f"\n/// Surface type distribution [(surface, cumulative_probability)]")
    print("const SURFACE_TYPE_DISTRIBUTION: &[(&str, f64)] = &[")
    cumulative = 0.0
    for surface, pct in surfaces:
        cumulative += pct / 100.0
        surface_display = surface if surface else "UNK"
        print(f'    ("{surface_display}", {cumulative:.4f}),  // {pct:.2f}%')
    print("];")



def main():
    # Determine database path
    if len(sys.argv) > 1:
        db_path = sys.argv[1]
    else:
        # Try default location
        home = Path.home()
        db_path = home / ".local" / "share" / "flight-planner" / "airports.db3"
        
        if not db_path.exists():
            # Try current directory
            db_path = Path("airports.db3")
            
        if not db_path.exists():
            print("Error: Could not find airports.db3")
            print("Usage: python analyze_airport_database.py [path/to/airports.db3]")
            sys.exit(1)
    
    print(f"Analyzing database: {db_path}\n")
    
    # Use a single database connection for all operations
    with sqlite3.connect(db_path) as conn:
        cursor = conn.cursor()
        analyze_airports(cursor)
        analyze_runways(cursor)
        generate_rust_code(cursor)
    
    print("\n" + "=" * 80)
    print("Analysis complete!")
    print("=" * 80)


if __name__ == "__main__":
    main()
