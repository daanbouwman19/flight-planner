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

## Setup and Installation

### 1. Prerequisites

Before you can run Flight Planner, you need to provide an airports database.

- **Airports Database**: The application requires an `airports.db3` file, which is **not included**. You can typically generate this file from your flight simulator's scenery data using various third-party tools.
- **Rust Toolchain**: If you are building from source, you need to install Rust from [rust-lang.org](https://www.rust-lang.org/).

### 2. Place Data Files

Copy your `airports.db3` file into the application's data directory. The location depends on your operating system:
-   **Linux**: `~/.local/share/flight-planner/`
-   **Windows**: `%APPDATA%\FlightPlanner\` (e.g., `C:\Users\YourUser\AppData\Roaming\FlightPlanner`)
-   **macOS**: `~/Library/Application Support/flight-planner/`

Alternatively, you can place the `airports.db3` file in the root of the project directory for development.

**(Optional) Aircraft Data:**
You can create an `aircrafts.csv` file in the same data directory to automatically import your aircraft data on first run. The CSV file should have the following columns: `manufacturer`, `variant`, `icao_code`, `flown`, `aircraft_range`, `category`, `cruise_speed`, `date_flown`, `takeoff_distance`.

### 3. Installation Methods

#### From Installers (Recommended for Users)
-   **Windows**: A Windows Installer (`.msi`) can be built using the `build_msi.bat` script. This requires the [WiX Toolset](https://wixtoolset.org/) to be installed.
-   **Linux**: An installation script is provided. Run the following commands from the project root to build and install the application system-wide:
    ```bash
    make
    sudo make install
    ```

#### From Source (for Developers)
1.  **Clone the repository:**
    ```bash
    git clone https://github.com/your-username/flight-planner.git
    cd flight-planner
    ```
2.  **Build the project:**
    ```bash
    cargo build --release
    ```

## Usage

You can run the application in two modes: GUI or CLI.

### GUI Mode

To run the graphical user interface, execute the following command from the project root:

```bash
cargo run
```

### CLI Mode

To run the command-line interface, use the `--cli` flag:

```bash
cargo run -- --cli
```
The CLI will present a menu of options for generating routes and managing your aircraft.

## Documentation

This repository is thoroughly documented. To generate and view the documentation for the codebase, run:

```bash
cargo doc --open
```

This will build the Rustdoc documentation and open it in your web browser.