# Flight Planner

Flight Planner is a desktop application designed for flight simulation enthusiasts. It helps users discover new routes by generating random flights based on a comprehensive database of airports and a user-provided fleet of aircraft. The application considers aircraft range and runway requirements to suggest realistic and flyable routes.

It features both a graphical user interface (GUI) and a command-line interface (CLI) for flexibility.

## Features

- **Random Route Generation**: Generate random routes between airports, with options to specify a departure airport.
- **Aircraft-Specific Routes**: Find routes suitable for a specific aircraft's range and performance characteristics.
- **Not-Flown Aircraft Routes**: Discover new destinations using aircraft you haven't flown yet.
- **Database-Driven**: Utilizes an airport database for accurate airport and runway information.
- **Aircraft Data Import**: Import your aircraft data from a CSV file.
- **Flight History**: Automatically tracks completed flights.
- **Statistics**: View detailed statistics about your flight history, including total distance, most flown aircraft, and favorite airports.
- **Cross-Platform**: Built with Rust and `eframe` for support on Windows, macOS, and Linux.
- **Dual Interface**: Usable as a rich GUI application or a fast command-line tool.

## Installation and Setup (for Users)

For most users, the recommended way to install Flight Planner is to download the latest installer from the [**GitHub Releases**](https://github.com/daanbouwman19/flight-planner/releases) page.

After installing, you will need to provide two data files:

1.  **Airports Database (Required)**:
    -   The application requires an `airports.db3` file, which is **not included**. You can typically generate this file from your flight simulator's scenery data using various third-party tools.
    -   Place this file in the application's data directory:
        -   **Linux**: `~/.local/share/flight-planner/`
        -   **Windows**: `%APPDATA%\FlightPlanner\` (e.g., `C:\Users\YourUser\AppData\Roaming\FlightPlanner`)
        -   **macOS**: `~/Library/Application Support/flight-planner/`

2.  **Aircraft Data (Optional)**:
    -   You can create an `aircrafts.csv` file in the same data directory to automatically import your aircraft data on the first run.
    -   The CSV file should have the following columns: `manufacturer`, `variant`, `icao_code`, `flown`, `aircraft_range`, `category`, `cruise_speed`, `date_flown`, `takeoff_distance`.

## Building from Source (for Developers)

If you wish to build the application from the source code, follow these steps:

1.  **Install Rust:** If you don't have it installed, get the Rust toolchain from [rust-lang.org](https://www.rust-lang.org/).
2.  **Clone the repository:**
    ```bash
    git clone https://github.com/daanbouwman19/flight-planner.git
    cd flight-planner
    ```
3.  **Place Data Files:** For development, you can place your `airports.db3` and optional `aircrafts.csv` files in the root of the project directory.
4.  **Build and Run:**
    -   **GUI Mode**: `cargo run`
    -   **CLI Mode**: `cargo run -- --cli`

## Documentation

This repository is thoroughly documented. To generate and view the documentation for the codebase, run:

```bash
cargo doc --open
```

This will build the Rustdoc documentation and open it in your web browser.
