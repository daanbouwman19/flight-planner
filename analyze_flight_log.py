import re
from collections import defaultdict
from datetime import timedelta

def parse_duration(duration_str):
    # Extract number and unit
    match = re.match(r"([\d.]+)\s*(ns|µs|ms|s)", duration_str)
    if not match:
        return None
    
    value = float(match.group(1))
    unit = match.group(2)
    
    # Convert everything to microseconds for comparison
    if unit == "ns":
        return value / 1000.0  # Convert ns to µs
    elif unit == "µs":
        return value
    elif unit == "ms":
        return value * 1000.0
    elif unit == "s":
        return value * 1000000.0
    
    return None

def analyze_log_file(file_path):
    durations = defaultdict(list)
    
    with open(file_path, 'r') as f:
        for line in f:
            if " in " not in line:
                continue
                
            # Extract operation and duration
            parts = line.strip().split(" - ")
            if len(parts) != 2:
                continue
                
            operation = parts[1].split(" in ")[0]
            duration_str = parts[1].split(" in ")[1]
            
            duration_us = parse_duration(duration_str)
            if duration_us is not None:
                durations[operation].append(duration_us)
    
    # Calculate averages and sort
    averages = []
    for operation, values in durations.items():
        avg_us = sum(values) / len(values)
        averages.append((operation, avg_us, len(values)))
    
    averages.sort(key=lambda x: x[1], reverse=True)
    
    # Print results
    print("\nOperation Statistics (ordered by average duration):")
    print("-" * 80)
    print(f"{'Operation':<30} {'Avg Duration':>15} {'Count':>10}")
    print("-" * 80)
    
    for operation, avg_us, count in averages:
        # Format duration nicely
        if avg_us >= 1000:
            duration_str = f"{avg_us/1000:.2f} ms"
        else:
            duration_str = f"{avg_us:.2f} µs"
            
        print(f"{operation:<30} {duration_str:>15} {count:>10}")

if __name__ == "__main__":
    import sys
    
    if len(sys.argv) != 2:
        print("Usage: python analyze_flight_log.py <log_file_path>")
        sys.exit(1)
        
    log_file_path = sys.argv[1]
    
    try:
        analyze_log_file(log_file_path)
    except Exception as e:
        print(f"An error occurred while reading the log file: {e}")
        sys.exit(1)