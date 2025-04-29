use clap::Parser;
use ctrlc;
use notify_rust::Notification;
use std::fs::File;
use std::io::{self, Write};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::{thread, time};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct PomodoroConfig {
    #[arg(short = 'n', long, value_delimiter = ',')]
    sessions: Option<Vec<u64>>,
    #[arg(short = 's', long, default_value_t = 5)]
    short_break: u64,

    /// Long break length in minutes
    #[arg(short = 'l', long, default_value_t = 20)]
    long_break: u64,

    /// Export summary as CSV to this file
    #[arg(short, long)]
    export_csv: Option<String>,

    /// Use a CSV file as a session database
    #[arg(long)]
    db_csv: Option<String>,
}

struct PomodoroState {
    session_count: usize,
    completed_sessions: Vec<(usize, u64)>,
}

fn beep() {
    print!("\x07");
    io::stdout().flush().ok();
}

fn notify(label: &str) {
    if Notification::new()
        .summary("Pomodoro")
        .body(label)
        .show()
        .is_err()
    {
        beep();
    }
}

fn clear_terminal() {
    print!("\x1B[2J\x1B[1;1H");
    io::stdout().flush().ok();
}

// The countdown function handles the timer and interruption logic.
fn countdown(minutes: u64, label: &str, running: &Arc<AtomicBool>) -> bool {
    let mut seconds: u64 = minutes * 60;
    while seconds > 0 && running.load(Ordering::SeqCst) {
        let min: u64 = seconds / 60;
        let sec: u64 = seconds % 60;
        print!("\r{}: {:02}:{:02} remaining...", label, min, sec);
        io::stdout().flush().unwrap();
        thread::sleep(time::Duration::from_secs(1));
        seconds -= 1;
    }
    if running.load(Ordering::SeqCst) {
        println!("\r{} done!                ", label);
        notify(label);
        true
    } else {
        println!("\nInterrupted!");
        false
    }
}

fn main() {
    let config: PomodoroConfig = PomodoroConfig::parse();
    let session_lengths: Vec<u64> = config.sessions.unwrap_or(vec![10, 15, 20, 25]);
    if session_lengths.iter().any(|&x| x == 0) || config.short_break == 0 || config.long_break == 0
    {
        eprintln!("Session and break lengths must be positive integers.");
        std::process::exit(1);
    }
    let running: Arc<AtomicBool> = Arc::new(AtomicBool::new(true));
    let r: Arc<AtomicBool> = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    let mut state: PomodoroState = PomodoroState {
        session_count: 0,
        completed_sessions: Vec::new(),
    };

    if let Some(ref db_path) = config.db_csv {
        if let Ok(file) = File::open(db_path) {
            let mut rdr: csv::Reader<File> = csv::Reader::from_reader(file);
            for result in rdr.records() {
                if let Ok(record) = result {
                    if let (Ok(i), Ok(len)) = (
                        record.get(0).unwrap_or("0").parse(),
                        record.get(1).unwrap_or("0").parse(),
                    ) {
                        state.completed_sessions.push((i, len));
                        state.session_count = i;
                    }
                }
            }
        }
    }

    let mut csv_writer: Option<csv::Writer<File>> = config
        .export_csv
        .as_ref()
        .map(|path: &String| csv::Writer::from_writer(File::create(path).unwrap()));
    let mut db_csv_writer: Option<csv::Writer<File>> =
        config.db_csv.as_ref().map(|path: &String| {
            csv::Writer::from_writer(
                std::fs::OpenOptions::new()
                    .append(true)
                    .create(true)
                    .open(path)
                    .unwrap(),
            )
        });

    loop {
        let length: u64 = if state.session_count < session_lengths.len() {
            session_lengths[state.session_count]
        } else {
            *session_lengths.last().unwrap()
        };
        clear_terminal();
        println!(
            "\nSession {}: Work for {} minutes!",
            state.session_count + 1,
            length
        );
        let completed: bool = countdown(length, "Work", &running);
        if !completed {
            break;
        }
        state
            .completed_sessions
            .push((state.session_count + 1, length));
        if let Some(ref mut db_writer) = db_csv_writer {
            let _ = db_writer
                .write_record(&[(state.session_count + 1).to_string(), length.to_string()]);
            let _ = db_writer.flush();
        }
        state.session_count += 1;
        if state.session_count % 4 == 0 {
            println!("\nLong break: {} minutes!", config.long_break);
            if !countdown(config.long_break, "Long Break", &running) {
                break;
            }
        } else {
            println!("\nShort break: {} minutes!", config.short_break);
            if !countdown(config.short_break, "Short Break", &running) {
                break;
            }
        }
        println!("\nPress Enter to start the next session or Ctrl+C to exit.");
        let mut input: String = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            break;
        }
    }
    println!("\nPomodoro Summary:");
    for (i, len) in &state.completed_sessions {
        println!("Session {}: {} min", i, len);
    }
    if let Some(ref mut writer) = csv_writer {
        for (i, len) in &state.completed_sessions {
            let _ = writer.write_record(&[i.to_string(), len.to_string()]);
        }
        let _ = writer.flush();
        println!("Summary exported to CSV.");
    }
}
