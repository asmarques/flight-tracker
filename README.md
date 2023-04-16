# flight-tracker

[![Crate](https://img.shields.io/crates/v/flight-tracker.svg)](https://crates.io/crates/flight-tracker)
[![Documentation](https://docs.rs/flight-tracker/badge.svg)](https://docs.rs/flight-tracker)
![Build Status](https://github.com/asmarques/flight-tracker/workflows/CI/badge.svg)

Keep track of aircraft using ADSB messages.

## Usage

### As an application

Connect to a receiver which emits frames in AVR format:

`flight-tracker tcp 127.0.0.1 30002`

The received ADSB messages will be used to update a table of current aircraft positions:

```
  icao       call      alt    hdg    gs       vr           lat/lon  squawk  last
--------------------------------------------------------------------------------
39E687    AF1180      1750    269   192     -512  51.4655, -0.2349    0650     0
4CAFD3    FR1885     17800     25   341     -960  51.3663, -0.3822    2276     0
```

### As a library

If you want to integrate the tracker into your application, create a new instance:

```rust
let tracker = Tracker::new();
```

Continuously feed it with ADSB messages from a receiver:

```rust
loop {
    ...
    tracker.update_with_avr("*8D4840D6202CC371C32CE0576098;");
    ...
}
```

Get the list of current aircraft:

```rust
let interval = Duration::from_secs(60);
let aicraft_list = tracker.get_current_aircraft(&inverval);
```
