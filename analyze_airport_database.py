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


def get_percentile(cursor, table, column, percentile):
    """Get a specific percentile value from a column."""
    cursor.execute(f"""
        SELECT {column} 
        FROM {table} 
        ORDER BY {column} 
        LIMIT 1 
        OFFSET (SELECT CAST(COUNT(*) * ? AS INTEGER) FROM {table})
    """, (percentile,))
    result = cursor.fetchone()
    return result[0] if result else None


def analyze_airports(db_path):
    """Analyze airport statistics."""
    conn = sqlite3.connect(db_path)
    cursor = conn.cursor()
    
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
    
    conn.close()


def analyze_runways(db_path):
    """Analyze runway statistics."""
    conn = sqlite3.connect(db_path)
    cursor = conn.cursor()
    
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
        cursor.execute(f"""
            SELECT Length 
            FROM runways 
            WHERE Length > 0
            ORDER BY Length 
            LIMIT 1 
            OFFSET (SELECT CAST(COUNT(*) * ? AS INTEGER) FROM runways WHERE Length > 0)
        """, (p,))
        val = cursor.fetchone()[0]
        print(f"P{int(p*100):>2}:     {val:>8.0f} ft")
    print(f"Max:     {max_len:>8.0f} ft")
    
    # Runway width statistics
    print("\n" + "-" * 80)
    print("RUNWAY WIDTH DISTRIBUTION (feet)")
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
        cursor.execute(f"""
            SELECT Width 
            FROM runways 
            WHERE Width > 0
            ORDER BY Width 
            LIMIT 1 
            OFFSET (SELECT CAST(COUNT(*) * ? AS INTEGER) FROM runways WHERE Width > 0)
        """, (p,))
        val = cursor.fetchone()[0]
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
    
    conn.close()


def generate_rust_code(db_path):
    """Generate Rust code snippets for mock data based on statistics."""
    conn = sqlite3.connect(db_path)
    cursor = conn.cursor()
    
    print("\n" + "=" * 80)
    print("SUGGESTED RUST CODE FOR mock_data.rs")
    print("=" * 80)
    
    # Get percentiles for elevation
    percentiles = {}
    for p in [0.10, 0.25, 0.50, 0.60, 0.75, 0.85, 0.90, 0.95]:
        percentiles[p] = get_percentile(cursor, 'airports', 'Elevation', p)
    
    print("\n// Elevation distribution (based on real percentiles)")
    print("let elevation_rand = rng.random::<f64>();")
    print("let elevation = if elevation_rand < 0.25 {")
    print(f"    rng.random_range(-210..{int(percentiles[0.25])})  // P0-P25")
    print("} else if elevation_rand < 0.50 {")
    print(f"    rng.random_range({int(percentiles[0.25])}..{int(percentiles[0.50])})  // P25-P50")
    print("} else if elevation_rand < 0.75 {")
    print(f"    rng.random_range({int(percentiles[0.50])}..{int(percentiles[0.75])})  // P50-P75")
    print("} else if elevation_rand < 0.90 {")
    print(f"    rng.random_range({int(percentiles[0.75])}..{int(percentiles[0.90])})  // P75-P90")
    print("} else if elevation_rand < 0.95 {")
    print(f"    rng.random_range({int(percentiles[0.90])}..{int(percentiles[0.95])})  // P90-P95")
    print("} else {")
    print(f"    rng.random_range({int(percentiles[0.95])}..14422)  // P95-P100")
    print("};")
    
    # Runway count distribution
    print("\n// Runway count distribution (based on real data)")
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
    
    print("let runway_rand = rng.random::<f64>();")
    cumulative = 0.0
    for i, (count, pct) in enumerate(runway_dist):
        cumulative += pct / 100.0
        if i == 0:
            print(f"let num_runways = if runway_rand < {cumulative:.4f} {{")
        else:
            print(f"}} else if runway_rand < {cumulative:.4f} {{")
        print(f"    {count}  // {pct:.2f}%")
    print("} else {")
    print(f"    {runway_dist[-1][0]}  // fallback")
    print("};")
    
    # Runway length distribution
    print("\n// Runway length distribution (based on real percentiles)")
    length_percentiles = {}
    for p in [0.10, 0.25, 0.50, 0.75, 0.90, 0.95]:
        cursor.execute("""
            SELECT Length 
            FROM runways 
            WHERE Length > 0
            ORDER BY Length 
            LIMIT 1 
            OFFSET (SELECT CAST(COUNT(*) * ? AS INTEGER) FROM runways WHERE Length > 0)
        """, (p,))
        length_percentiles[p] = cursor.fetchone()[0]
    
    print("let length_rand = rng.random::<f64>();")
    print("let base_length = if length_rand < 0.25 {")
    print(f"    rng.random_range(80..{int(length_percentiles[0.25])})  // P0-P25")
    print("} else if length_rand < 0.50 {")
    print(f"    rng.random_range({int(length_percentiles[0.25])}..{int(length_percentiles[0.50])})  // P25-P50")
    print("} else if length_rand < 0.75 {")
    print(f"    rng.random_range({int(length_percentiles[0.50])}..{int(length_percentiles[0.75])})  // P50-P75")
    print("} else if length_rand < 0.90 {")
    print(f"    rng.random_range({int(length_percentiles[0.75])}..{int(length_percentiles[0.90])})  // P75-P90")
    print("} else {")
    print(f"    rng.random_range({int(length_percentiles[0.90])}..21119)  // P90-P100")
    print("};")
    
    # Runway width distribution
    print("\n// Runway width distribution (based on real percentiles)")
    width_percentiles = {}
    for p in [0.10, 0.25, 0.50, 0.75, 0.90]:
        cursor.execute("""
            SELECT Width 
            FROM runways 
            WHERE Width > 0
            ORDER BY Width 
            LIMIT 1 
            OFFSET (SELECT CAST(COUNT(*) * ? AS INTEGER) FROM runways WHERE Width > 0)
        """, (p,))
        width_percentiles[p] = cursor.fetchone()[0]
    
    print("let width_rand = rng.random::<f64>();")
    print("let width = if width_rand < 0.25 {")
    print(f"    rng.random_range(9..{int(width_percentiles[0.25])})  // P0-P25")
    print("} else if width_rand < 0.50 {")
    print(f"    rng.random_range({int(width_percentiles[0.25])}..{int(width_percentiles[0.50])})  // P25-P50")
    print("} else if width_rand < 0.75 {")
    print(f"    rng.random_range({int(width_percentiles[0.50])}..{int(width_percentiles[0.75])})  // P50-P75")
    print("} else {")
    print(f"    rng.random_range({int(width_percentiles[0.75])}..{int(width_percentiles[0.90])})  // P75-P90")
    print("};")
    
    # Surface type distribution
    print("\n// Surface type distribution (based on real data)")
    cursor.execute("""
        SELECT 
            Surface,
            CAST(COUNT(*) * 100.0 / (SELECT COUNT(*) FROM runways) AS REAL) as percentage
        FROM runways
        GROUP BY Surface
        ORDER BY COUNT(*) DESC
        LIMIT 5
    """)
    surfaces = cursor.fetchall()
    
    print("let surface_rand = rng.random::<f64>();")
    cumulative = 0.0
    for i, (surface, pct) in enumerate(surfaces):
        cumulative += pct / 100.0
        if i == 0:
            print(f"let surface = if surface_rand < {cumulative:.4f} {{")
        else:
            print(f"}} else if surface_rand < {cumulative:.4f} {{")
        surface_display = surface if surface else "UNK"
        print(f'    "{surface_display}"  // {pct:.2f}%')
    print("} else {")
    print(f'    "{surfaces[0][0]}"  // fallback')
    print("};")
    
    conn.close()


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
    
    analyze_airports(db_path)
    analyze_runways(db_path)
    generate_rust_code(db_path)
    
    print("\n" + "=" * 80)
    print("Analysis complete!")
    print("=" * 80)


if __name__ == "__main__":
    main()
