# CorelDRAW Version Converter Mechanics

This document details the mechanics, constants, formulas, and rules discovered in the original program `CDR convert version 3.5.1.exe` (`CDRFileMgr.exe`).

## 1. CorelDRAW Versions & Version Numbers

CorelDRAW files store their internal version as a number, where version 11 is 1100, 12 is 1200, X3 is 1300, X4 is 1400, X5 is 1500, X6 is 1600, X7 is 1700, X8 is 1800, 2019 is 2100, 2020 is 2200, 2021 is 2300, and 2022 is 2400.

| Version Name | Internal Number | Hex Value | Source |
|---|---|---|---|
| CorelDRAW 11 | 1100 | 0x044C | String `0x1407f5950`, function `0x1400a2632` |
| CorelDRAW 12 | 1200 | 0x04B0 | String table `0x1407f5970` |
| CorelDRAW X3 | 1300 | 0x0514 | Function `0x140130350` (check for 0x514) |
| CorelDRAW X4 | 1400 | 0x0578 | String `0x1407fd0a8`, function `0x140121f10` |
| CorelDRAW X5 | 1500 | 0x05DC | String `0x1407f5a60`, function `0x140130350` |
| CorelDRAW X6 | 1600 | 0x0640 | Constant 0x640 in function `0x1400a2410` |
| CorelDRAW X7 | 1700 | 0x06A4 | String `0x1407f4913` |
| CorelDRAW X8 | 1800 | 0x0708 | String `0x1407f9ab4`, function `0x1400d006e` |
| CorelDRAW 2017 | 1900 | 0x076C | Constant 0x76C in version dispatch |
| CorelDRAW 2018 | 2000 | 0x07D0 | Constant 0x7D0 in version dispatch |
| CorelDRAW 2019 | 2100 | 0x0834 | Menu shortcut `0x1407fd1a0` |
| CorelDRAW 2020 | 2200 | 0x0898 | Constant 0x898 in version dispatch |
| CorelDRAW 2021 | 2300 | 0x08FC | Constant 0x8FC in version dispatch (FourCC `CDRN`) |
| CorelDRAW 2022 | 2400 | 0x0960 | Constant 0x960 in version dispatch (FourCC `CDRO`) |
| CorelDRAW 2023 | 2450 | 0x0992 | CorelDRAW v24.x updates (FourCC `CDRO`) |
| CorelDRAW 2024 | 2500 | 0x09C4 | CorelDRAW v25.0 (FourCC `CDRP`) |
| CorelDRAW 2025 | 2600 | 0x0A28 | CorelDRAW v26.0 (FourCC `CDRQ`) |
| CorelDRAW 2026 | 2700 / 2720 | 0x0A8C | CorelDRAW v27.x (FourCC `CDRR`, `AppVersion` 2720) |

## 2. RIFF File Format and Chunk Headers

CorelDRAW files are either pure RIFF files (pre-X4) or ZIP containers containing `content/riffData.dat` or `content/root.dat` (X4 and later).

- `RIFF`: 0x46464952 (Function `0x140130350`)
- `LIST`: 0x5453494C (Function `0x140130350`)
- `DISP`: 0x50534944 (Function `0x140130350`)
- `vrsn`: 0x6E737276 (Function `0x140130350`)
- `outl`: 0x6C74756F (Function `0x140130350`)
- `sumi`: 0x696D7573 (Function `0x140130350`)
- `cmpr`: 0x72706D63 (Function `0x14010C040`)

### Version Detection Rule
In `sub_140130350`:
In the `vrsn` chunk:
- Read 16-bit word.
- If equal to `0xFFFF`, skip 6 bytes and read the subsequent 16-bit word as the real version number.
- Otherwise, the first 16-bit word is the version number.

## 3. Metadata and Cleaning Rules

In `metadata/metadata.xml` or `META-INF/metadata.xml` (modified by `sub_140121f10`):
- Namespace: `http://namespace.corel.com/Cdr/metadata` or `http://namespace.corel.com/cdr/metadata/1.0/`
- `cdr:CoreVersion` / `CoreVersion`: Updated to target version (e.g. `1400` for X4)
- `cdr:AppVersion` / `AppVersion`: Software version used to create/save file (e.g. `2720` for CorelDRAW 2026), updated to target version
- `ProductName`: CorelDRAW version name
- Higher version elements (`cdrinfo:ColorInfo`, `cdrinfo:CurveInfo`, `inClr:`, `inCrv:`, `inEff:`, etc.) are stripped when downgrading.

## 4. UI Preferences and Settings

From `sub_1400d006e`, `sub_1400d3900`, `sub_1400d46d0`:
- Default target version: `1400` (CDR X4)
- Output directory modes:
  - Mode 0: Same directory as source files
  - Mode 1: Specified subdirectory under source file
  - Mode 2: Specified absolute directory
- Suffix: `_x4` (default when converting to X4)
- Clean up cdr file junk: enabled by default (`true`)
- Fix outline id errors: enabled by default (`true`)
- Erase higher version info: enabled by default (`true`)
- Window dimensions: Width 780, Height 560 (estimated from layout sizers)
