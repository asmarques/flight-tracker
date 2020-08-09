use anyhow::Result;
use flight_tracker::Tracker;
use std::fmt;
use std::io;
use std::io::BufRead;
use std::io::BufReader;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use structopt::StructOpt;

const REFRESH_INTERVAL: Duration = Duration::from_secs(1);
const NA: &str = "N/A";

#[derive(StructOpt)]
#[structopt(about = "Track aircraft via ADSB")]
struct Cli {
    #[structopt(subcommand)]
    cmd: Command,
    #[structopt(
        name = "expire",
        help = "Number of seconds before removing stale entries",
        default_value = "60",
        short = "e",
        long = "expire"
    )]
    expire: u64,
}

#[derive(StructOpt)]
enum Command {
    #[structopt(about = "Read messages from stdin")]
    Stdin,
    #[structopt(about = "Read messages from a TCP server")]
    Tcp {
        #[structopt(help = "host")]
        host: String,
        #[structopt(help = "port", default_value = "30002")]
        port: u16,
    },
}

fn main() -> Result<()> {
    let args = Cli::from_args();
    let tracker = Arc::new(Mutex::new(Tracker::new()));
    let expire = Duration::from_secs(args.expire);
    let writer = write_output(tracker.clone(), expire);
    let reader = match args.cmd {
        Command::Stdin => read_from_stdin(tracker),
        Command::Tcp { host, port } => read_from_network(host, port, tracker),
    };

    reader.join().unwrap()?;
    writer.join().unwrap()?;

    Ok(())
}

fn read_from_stdin(tracker: Arc<Mutex<Tracker>>) -> JoinHandle<Result<()>> {
    thread::spawn(move || {
        let mut input = String::new();
        loop {
            let _ = io::stdin().read_line(&mut input)?;
            let mut tracker = tracker.lock().unwrap();
            let _ = tracker.update_with_avr(&input);
            input.clear();
        }
    })
}

fn read_from_network(
    host: String,
    port: u16,
    tracker: Arc<Mutex<Tracker>>,
) -> JoinHandle<Result<()>> {
    thread::spawn(move || {
        let stream = TcpStream::connect((host.as_str(), port))?;
        let mut reader = BufReader::new(stream);
        let mut input = String::new();
        loop {
            let _ = reader.read_line(&mut input)?;
            let mut tracker = tracker.lock().unwrap();
            let _ = tracker.update_with_avr(&input);
            input.clear();
        }
    })
}

fn write_output(tracker: Arc<Mutex<Tracker>>, expire: Duration) -> JoinHandle<Result<()>> {
    thread::spawn(move || loop {
        thread::sleep(REFRESH_INTERVAL);
        let tracker = tracker.lock().unwrap();
        print_ascii_table(&tracker, &expire);
    })
}

fn fmt_value<T: fmt::Display>(value: Option<T>, precision: usize) -> String {
    value
        .map(|v| format!("{:.1$}", v, precision))
        .unwrap_or_else(|| NA.to_string())
}

fn print_ascii_table(tracker: &Tracker, expire: &Duration) {
    let aircraft_list = tracker.get_current_aircraft(expire);
    // Clear screen
    print!("\x1B[2J\x1B[H");
    println!(
        "{:>6} {:>10} {:>8} {:>6} {:>5} {:>8} {:>17} {:>5}",
        "icao", "call", "alt", "hdg", "gs", "vr", "lat/lon", "last"
    );
    println!("{}", "-".repeat(72));
    for aircraft in aircraft_list {
        println!(
            "{:>6} {:>10} {:>8} {:>6} {:>5} {:>8} {:>8},{:>8} {:>5}",
            aircraft.icao_address,
            aircraft.callsign.clone().unwrap_or_else(|| NA.to_string()),
            fmt_value(aircraft.altitude, 0),
            fmt_value(aircraft.heading, 0),
            fmt_value(aircraft.ground_speed, 0),
            fmt_value(aircraft.vertical_rate, 0),
            fmt_value(aircraft.latitude, 4),
            fmt_value(aircraft.longitude, 4),
            aircraft.last_seen.elapsed().unwrap_or_default().as_secs(),
        );
    }
}
