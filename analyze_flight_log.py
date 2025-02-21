import re
from collections import defaultdict
import numpy as np
import matplotlib.pyplot as plt
import argparse
from datetime import datetime
import logging

DURATION_FORMAT = "{:.2f} {}"
COLUMN_FORMAT = "{:<30} {:>15} {:>15} {:>15} {:>10}"
HEADER_FORMAT = "\nOperation Statistics (ordered by average duration):\n" + "-" * 120 + "\n" + COLUMN_FORMAT.format("Operation", "Avg Duration", "Std Dev", "90th Pct", "Count") + "\n" + "-" * 120

# Pre-compile the regular expression pattern
DURATION_PATTERN = re.compile(r"([\d.]+)\s*(ns|µs|us|ms|s|m)?s?")

def format_duration(duration_microseconds: float) -> str:
    """Helper function to format duration."""
    if duration_microseconds >= 1000:
        return DURATION_FORMAT.format(duration_microseconds / 1000, "ms")
    return DURATION_FORMAT.format(duration_microseconds, "µs")

def parse_duration(duration_str: str) -> float | None:
    """Parses a duration string into microseconds."""
    duration_str = duration_str.replace("main() is: ", "").strip()
    duration_str = duration_str.replace("Âµs", "µs")
    match = DURATION_PATTERN.match(duration_str)  # Use pre-compiled pattern
    if not match:
        logging.error(f"Failed to parse duration: {duration_str}")
        return None

    value = float(match.group(1))
    unit = match.group(2) or "µs"

    if unit == "ns":
        return value / 1000.0
    elif unit in ["µs", "us"]:
        return value
    elif unit == "ms":
        return value * 1000.0
    elif unit == "s":
        return value * 1000000.0
    elif unit == "m":
        return value * 60 * 1000000.0
    return None

def analyze_log_file(file_path: str, generate_chart: bool = False) -> None:
    """Analyzes a log file and generates operation statistics."""
    durations = defaultdict(list)
    timestamps = defaultdict(list)  # Add timestamps dictionary

    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            for line in f:
                parts = line.strip().split(" - ")
                if len(parts) != 2:
                    continue

                # Fix timestamp parsing
                timestamp_str = parts[0]  # Get first element
                try:
                    timestamp = datetime.fromisoformat(timestamp_str.replace("Z", "+00:00"))
                except ValueError as e:
                    logging.error(f"Invalid timestamp format: {timestamp_str} - Error: {e}")
                    continue

                # Fix operation and duration parsing
                operation_parts = parts[1].split(" in ")
                if len(operation_parts) != 2:
                    continue

                operation = operation_parts[0]
                duration_str = operation_parts[1]

                duration_microseconds = parse_duration(duration_str)
                if duration_microseconds is not None:
                    timestamps[operation].append(timestamp)  # Store timestamp
                    durations[operation].append(duration_microseconds)

    except FileNotFoundError:
        logging.error(f"Log file not found at {file_path}")
        return
    except Exception:
        logging.exception("An error occurred while reading the log file")
        return

    averages = []
    for operation, values in durations.items():
        # Sort values by timestamp
        sorted_pairs = sorted(zip(timestamps[operation], values))
        sorted_timestamps, sorted_values = zip(*sorted_pairs) if values else ([], [])
        
        avg_us = np.mean(sorted_values) if sorted_values else 0
        std_dev_us = np.std(sorted_values) if sorted_values else 0
        p90_us = np.percentile(sorted_values, 90) if sorted_values else 0
        averages.append((operation, avg_us, std_dev_us, p90_us, len(sorted_values), sorted_timestamps, sorted_values))

    if generate_chart:
        generate_bar_chart(averages)

    averages.sort(key=lambda x: x, reverse=True)  # Sort by average duration

    print(HEADER_FORMAT)

    for operation, average_duration_microseconds, standard_deviation_microseconds, p90_duration_microseconds, count, _, _ in averages:
        print(COLUMN_FORMAT.format(
            operation,
            format_duration(average_duration_microseconds),
            format_duration(standard_deviation_microseconds),
            format_duration(p90_duration_microseconds),
            count
        ))

def generate_bar_chart(averages: list[tuple[str, float, float, float, int, list, list]]) -> None:
    """Generates a bar chart of operation durations with standard deviation."""
    operations = [item[0] for item in averages]
    avg_durations = [item[1] for item in averages]
    std_devs = [item[2] for item in averages]

    plt.figure(figsize=(12, 6))
    plt.bar(operations, avg_durations, yerr=std_devs, capsize=5)
    plt.xticks(rotation=45, ha='right')
    plt.xlabel("Operations")
    plt.ylabel("Average Duration (µs)")
    plt.title("Average Operation Durations with Standard Deviation")
    plt.tight_layout()
    plt.show()

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Analyze a log file.")
    parser.add_argument("log_file_path", help="Path to the log file.")
    parser.add_argument("--chart", action="store_true", help="Generate a chart.")
    args = parser.parse_args()

    analyze_log_file(args.log_file_path, args.chart)