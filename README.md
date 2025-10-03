# Flight Planner

Flight Planner is a desktop application designed for flight simulation enthusiasts. It helps users discover new routes by generating random flights based on a comprehensive database of airports and a user-provided fleet of aircraft. The application considers aircraft range and runway requirements to suggest realistic and flyable routes.

It features both a graphical user interface (GUI) and a command-line interface (CLI) for flexibility.

## Features

- **Random Route Generation**: Generate random routes between airports, with options to specify a departure airport.
- **Aircraft-Specific Routes**: Find routes suitable for a specific aircraft's range and performance characteristics.
- **Not-Flown Aircraft Routes**: Discover new destinations using aircraft you haven't flown yet.
- **Database-Driven**: Utilizes an airport database for accurate airport and runway information.
- **Aircraft Fleet Management**: Import your aircraft fleet from a CSV file.
- **Flight History**: Automatically tracks completed flights.
- **Statistics**: View detailed statistics about your flight history, including total distance, most flown aircraft, and favorite airports.
- **Cross-Platform**: Built with Rust and `eframe` for support on Windows, macOS, and Linux.
- **Dual Interface**: Usable as a rich GUI application or a fast command-line tool.

## Prerequisites

Before you can run Flight Planner, you need to provide an airports database.

- **Airports Database**: The application requires an `airports.db3` file. This file is **not included** with the application. You can typically generate this file from your flight simulator's scenery data using tools like Little Navmap.

## Setup and Installation

1.  **Clone the repository:**
    ```bash
    git clone https://github.com/your-username/flight-planner.git
    cd flight-planner
    ```

2.  **Install Rust:** If you don't have Rust installed, get it from [rust-lang.org](https://www.rust-lang.org/).

3.  **Place the Database File:**
    Copy your `airports.db3` file into the application's data directory. The location depends on your operating system:
    -   **Linux**: `~/.local/share/flight-planner/`
    -   **Windows**: `%APPDATA%\FlightPlanner\` (e.g., `C:\Users\YourUser\AppData\Roaming\FlightPlanner`)
    -   **macOS**: `~/Library/Application Support/flight-planner/`

    Alternatively, you can place the `airports.db3` file in the root of the project directory for development.

4.  **(Optional) Aircraft Data:**
    You can create an `aircrafts.csv` file in the same data directory to automatically import your aircraft fleet the first time you run the application. The CSV file should have the following columns: `manufacturer`, `variant`, `icao_code`, `flown`, `aircraft_range`, `category`, `cruise_speed`, `date_flown`, `takeoff_distance`.

5.  **Build the project:**
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

## Building for Distribution

-   **Linux**: The `Makefile` contains targets for building and installing the application system-wide.
    ```bash
    make
    sudo make install
    ```

-   **Windows**: A `build_msi.bat` script is provided to create a Windows Installer (MSI) package. This requires the [WiX Toolset](https://wixtoolset.org/) to be installed.

## Documentation

This repository is thoroughly documented. To generate and view the documentation for the codebase, run:

```bash
cargo doc --open
```

This will build the Rustdoc documentation and open it in your web browser.