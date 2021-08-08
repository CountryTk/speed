use std::env;
use std::fs::read;
use std::thread;
use std::time::{Duration, Instant};

use argparse::{ArgumentParser, Store, StoreTrue};
use std::io::{self, stdin, stdout, BufRead, Read, Write};
use termion::event::{Event, Key};
use termion::input::TermRead;
use termion::raw::IntoRawMode;

fn main() {

    let mut possible_file_size: f32 = 0.0;
    let mut file_name = String::new();
    {
        // this block limits scope of borrows by ap.refer() method
        let mut ap = ArgumentParser::new();
        ap.set_description("See file upload speeds");
        ap.refer(&mut file_name).add_option(
            &["--file", "-f"],
            Store,
            "File to track its download speed",
        );
        ap.refer(&mut possible_file_size).add_option(
            &["--size", "-s"],
            Store,
            "Known file end size in MB",
        );
        ap.parse_args_or_exit();
    }

    let mut average_speed: Vec<f32> = Vec::new();
    let mut total_mb_recorded: f32 = 0.0;

    let mut knows_size = false;

    if possible_file_size != 0.0 {
        knows_size = true;
    }

    println!("Enter final file size in MB (if known): ");
    let mut prev: f32 = 0.0;
    let mut break_counter: i32 = 0;

    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();
    
        print!("{}{}", termion::cursor::Goto(1, 1), termion::clear::All);

        stdout.flush().unwrap();

    loop {
        let bytes: Vec<u8> = read(&file_name).unwrap();
        let mb = (bytes.len() as f32) / 1000000.0;

        if prev != 0.0 {
            let avg_speed = (mb - prev) / 1.0;
            if avg_speed == 0.0 {
                if break_counter == 1 {
                    print!("{}{}", termion::cursor::Goto(1, 1), termion::clear::All);
                    break;
                } else {
                    break_counter += 1;
                }
            }
            average_speed.push(avg_speed);
            prev = mb;
            total_mb_recorded = prev;
            print!("{}{}", termion::cursor::Goto(1, 1), termion::clear::All);

            if knows_size {
                print!(
                        "{}Current speed: {} MB/s, file size: {} MB, estimated time until done: {} seconds",
                        termion::cursor::Goto(1, 1),
                        avg_speed,
                        mb,
                        (possible_file_size-mb) / avg_speed,
                    );
            } else {
                print!(
                    "{}Current speed: {} MB/s, file size: {} MB",
                    termion::cursor::Goto(1, 1),
                    avg_speed,
                    mb
                );
            }
            stdout.flush().unwrap();
        } else {
            prev = mb;
        }

        thread::sleep(Duration::from_millis(1000));
    }

    let avg: f32 = average_speed.iter().sum::<f32>() / (average_speed.len() as f32);

    println!(
        "Average speed: {} MB/s, Total MB transferred: {}",
        avg, total_mb_recorded
    );
}
