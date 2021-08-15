use argparse::{ArgumentParser, Store};
use std::fs::read;
use std::io::{stdin, stdout, Write};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

#[derive(Clone, Debug)]
struct FileData {
    speed: f32,
    transferred_size: f32,
    previous_size: f32,
}

impl FileData {
    fn set_speed(&mut self, new_speed: f32) {
        self.speed = new_speed;
    }

    fn set_transferred(&mut self, new_size: f32) {
        self.transferred_size = new_size;
    }

    fn set_prev(&mut self, new_prev: f32) {
        self.previous_size = new_prev;
    }
}

fn read_file_get_speed(
    sender: Sender<FileData>,
    file_name: &String,
    previous_file_size: f32,
    transferred_size: f32,
) -> FileData {
    let mut prev: f32 = previous_file_size;
    let mut average_speed: f32 = 0.0;
    let mut total_mb_recorded: f32 = transferred_size;

    let bytes: Vec<u8> = read(file_name).unwrap();
    let mb = (bytes.len() as f32) / 1000000.0;
    if prev != 0.0 {
        average_speed = (mb - prev) / 1.0;
        prev = mb;
        total_mb_recorded = prev;
    } else {
        prev = mb;
    }

    thread::sleep(Duration::from_millis(1000));

    let data = FileData {
        speed: average_speed,
        transferred_size: total_mb_recorded,
        previous_size: prev,
    };

    sender.send(data.clone()).unwrap();

    data
}

fn main() {
    let mut possible_file_size: f32 = 0.0;
    let mut file_name = String::new();
        
    let global_speed: Arc<Mutex<FileData>> = Arc::new(Mutex::new(FileData {
        speed: 0.0,
        transferred_size: 0.0,
        previous_size: 0.0,
    }));

    {
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

    let running = Arc::new(Mutex::new(true));
    let first_iteration = Arc::new(Mutex::new(true));
    let break_counter = Arc::new(Mutex::new(0));
    let file_size_known = Arc::new(Mutex::new(false));

    if possible_file_size != 0.0 {
        let mut size_known = file_size_known.lock().unwrap();
        *size_known = true;
    }

    while *running.lock().unwrap() {
        let fil = file_name.clone();

        let stdin = stdin();
    
        let mut stdout = stdout().into_raw_mode().unwrap();

        print!("{}", termion::clear::All);
        print!("Loading...");
        stdout.flush().unwrap();

        let global_speed = Arc::clone(&global_speed);
        let global_iteration = Arc::clone(&first_iteration);
        let global_break_counter = Arc::clone(&break_counter);
        let global_size_known = Arc::clone(&file_size_known);

        thread::spawn(move || {
            let (child_tx, child_rx): (Sender<FileData>, Receiver<FileData>) = mpsc::channel();
            let running = Arc::new(Mutex::new(true));
     
            while *running.lock().unwrap() {
                let mut ok_children = Vec::new();
                let child_thread_tx = child_tx.clone();
                let inner_file = fil.clone();
                
                let global_speed = Arc::clone(&global_speed);
                let global_iteration = Arc::clone(&global_iteration);
                let global_break_counter = Arc::clone(&global_break_counter);
                let global_size_known = Arc::clone(&global_size_known);

                let child = thread::spawn(move || {

                    let data: FileData;
                    let mut speed = global_speed.lock().unwrap();
                    let mut first_iteration = global_iteration.lock().unwrap();

                    if *first_iteration {
                        *first_iteration = false;
                        data = read_file_get_speed(child_thread_tx, &inner_file, 0.0, 0.0);
                    } else {
                        data = read_file_get_speed(
                            child_thread_tx,
                            &inner_file,
                            speed.previous_size,
                            speed.transferred_size,
                        );
                    }

                    if data.speed == 0.0 {
                        let mut break_counter = global_break_counter.lock().unwrap();
                        *break_counter += 1;

                        if *break_counter == 3 {
                            print!("{}{}", termion::cursor::Goto(1, 1), termion::clear::All);
                            print!("{}", termion::cursor::Show);

                           return true;
                        }
                    }

                    speed.set_speed(data.speed);
                    speed.set_transferred(data.transferred_size);
                    speed.set_prev(data.previous_size);

                    let size_known = global_size_known.lock().unwrap();
                    println!("{}", termion::clear::All);

                    if *size_known {

                        print!(
                            "{}{}{}",
                            termion::cursor::Hide,
                            termion::cursor::Goto(1,1),
                            termion::clear::CurrentLine);
                        if speed.speed != 0.0 {
                        println!("{:.1}MB/{}MB {:.1}MB/s, Time left: {:.1} seconds, file: {}", speed.transferred_size, possible_file_size, speed.speed, (possible_file_size-speed.transferred_size)/speed.speed, &inner_file);
                        } else {
                            println!("Loading...");
                        }
                    } else {
                        print!(
                            "{}{}{}",
                            termion::cursor::Hide,
                            termion::cursor::Goto(1, 1),
                            termion::clear::CurrentLine
                        );
                        println!("{:.1}MB, {}{:.1}{} MB/s, file: {}", speed.transferred_size, termion::color::Fg(termion::color::Green), speed.speed, termion::color::Fg(termion::color::Reset), &inner_file);
                        
                    }
                    return false;
                });

                ok_children.push(child);

                child_rx.recv().unwrap();

                for child in ok_children {
                    let status = child.join().unwrap();
                    if status {
                        *running.lock().unwrap() = false;
                        print!("{}{}", termion::clear::All, termion::cursor::Hide);
                        println!("Download finished... press Ctrl+C to exit");
                    }
                }

            }
        });
        for key in stdin.keys() {
            match key.unwrap() {
                Key::Ctrl('z') => {
                    let mut running = running.lock().unwrap();
                    *running = false;

                    print!("{}", termion::cursor::Show);
                    break;
                }
                Key::Ctrl('c') => {
                    let mut running = running.lock().unwrap();
                    *running = false;

                    print!("{}", termion::cursor::Show);
                    break;
                }
                _ => {}
            }
        }
        print!("{}{}", termion::cursor::Goto(1, 1), termion::clear::All);
        stdout.flush().unwrap();
    }
}
