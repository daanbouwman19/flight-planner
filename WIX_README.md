# WiX MSI Installer for Flight Planner

This directory contains the WiX Toolset configuration for creating a professional Windows MSI installer for Flight Planner.

## Requirements

- **WiX Toolset v6.0.2+**: Download from https://wixtoolset.org/
- **Rust toolchain**: For building the release version
- **Windows**: WiX only runs on Windows

## Installation

1. **Download and install WiX v6.0.2+**
   - Go to https://wixtoolset.org/
   - Download the latest installer
   - Run the installer
   - WiX v6+ uses modern .NET and has updated syntax

2. **Verify installation**:
   ```cmd
   wix --version
   ```
   This should show WiX v6.0.2 or later

## Quick Start

1. **Build the MSI installer**:
   ```cmd
   build_msi.bat
   ```

2. **Find your installer**: `dist/FlightPlannerSetup.msi`

3. **Test installation**: Double-click the MSI file

## Manual Build

If you prefer manual steps:

```cmd
# Build Rust application
cargo build --release

# Build MSI with WiX v6 (single command)
wix build FlightPlanner.wxs -out dist/FlightPlannerSetup.msi
```

## MSI Installer Features

- ✅ **Professional MSI format** - Microsoft standard installer
- ✅ **Program Files installation** - Proper Windows directory structure
- ✅ **Start Menu shortcuts** - Application launcher in Start Menu
- ✅ **Add/Remove Programs** - Proper Windows uninstaller integration
- ✅ **Upgrade support** - Can upgrade existing installations
- ✅ **Administrative privileges** - Handles UAC correctly
- ✅ **GUI wizard** - Professional installation interface
- ✅ **Component-based** - Proper Windows Installer architecture

## File Structure

- `FlightPlanner.wxs` - Main WiX source file
- `build_msi.bat` - Automated build script
- `dist/` - Output directory for MSI installer

## What Gets Installed

```
C:\Program Files\Flight Planner\
├── flight_planner.exe    # Main application
└── aircrafts.csv         # Aircraft database
```

## Customization

### Change Installation Directory
Edit the `INSTALLFOLDER` directory in `FlightPlanner.wxs`:

```xml
<Directory Id="INSTALLFOLDER" Name="Your App Name" />
```

### Add More Files
Add new `<File>` elements in the appropriate `<Component>`:

```xml
<File Id="YourFile" Source="path\to\your\file.txt" />
```

### Change Product Information
Update the `<Package>` element in `FlightPlanner.wxs`:

```xml
<Package Name="Your App Name" 
         Version="1.0.0" 
         Manufacturer="Your Name"
         UpgradeCode="{PUT-YOUR-GUID-HERE}" />
```

## Troubleshooting

**"wix command not found" or "candle.exe not found"**
- Install WiX Toolset v6.0.2+ from https://wixtoolset.org/
- Restart command prompt after installation
- Verify with: wix --version

**"cargo build failed"**
- Make sure Rust is properly installed
- Check for compilation errors
- Ensure all dependencies are available

**"WiX compilation failed"**
- Check that all source files exist
- Verify file paths in FlightPlanner.wxs
- Look for XML syntax errors

## Notes

- The MSI excludes `airports.db3` as users provide this file themselves
- The application will guide users on `airports.db3` placement
- MSI installers require administrator privileges for installation
- The installer supports both installation and upgrade scenarios

## Distribution

The resulting `FlightPlannerSetup.msi` file can be:
- Distributed via email or download
- Deployed via Group Policy in corporate environments
- Signed with a code signing certificate for trust
- Published to software repositories

---

**WiX Toolset**: Microsoft's official free installer creation tool  
**MSI Format**: Windows standard for professional software installation
