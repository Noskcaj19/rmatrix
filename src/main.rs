extern crate clap;
extern crate rand;
extern crate termbuf;

use clap::{App as ClapApp, Arg};

use termbuf::termion::async_stdin;
use termbuf::termion::event::Key;
use termbuf::termion::input::TermRead;
use termbuf::{Color, Style};
use termbuf::{TermBuf, TermSize};

use rand::prelude::*;
use rand::prng::XorShiftRng;
use std::thread;
use std::time::Duration;

struct Options {
    char_type: CharType,
    delay: usize,
    bolding: bool,
}

enum CharType {
    Ascii,
    Kana,
}

struct App {
    termbuf: TermBuf,
    size: TermSize,
    streams: Vec<TextStream>,
    rng: XorShiftRng,
    opts: Options,
}

#[derive(Clone, Debug)]
pub struct TextStream {
    pub x: usize,
    pub y: usize,
    pub len: usize,
    pub alive: bool,
}

impl TextStream {
    pub fn new(x: usize, len: usize) -> TextStream {
        TextStream {
            x,
            y: 0,
            len,
            alive: true,
        }
    }
}

fn random_ascii(rng: &mut XorShiftRng) -> char {
    rng.gen_range(b'!', b'}') as char
}

fn random_kana(rng: &mut XorShiftRng) -> char {
    let x = &[rng.gen_range(0xff62, 0xff9e)];
    std::char::decode_utf16(x.iter().cloned())
        .next()
        .unwrap()
        .unwrap()
}

impl App {
    fn draw_streams(&mut self) {
        // Borrowck trick
        let mut rng = &mut self.rng;
        for stream in self.streams.iter_mut() {
            let ch = if let CharType::Ascii = self.opts.char_type {
                random_ascii(&mut rng)
            } else {
                random_kana(&mut rng)
            };
            // Print random character in stream
            self.termbuf
                .char_builder(ch, stream.x, stream.y)
                .fg(Color::White)
                .build();

            // TODO: magic number
            if rng.gen::<u8>() < 60u8 && self.opts.bolding {
                self.termbuf
                    .set_cell_style(Style::Bold, stream.x, stream.y.saturating_sub(1));
            }
            self.termbuf
                .set_cell_fg(Color::Green, stream.x, stream.y.saturating_sub(1));

            // Clear stream
            if stream.y >= stream.len {
                self.termbuf.set_char(' ', stream.x, stream.y - stream.len);
            };
            stream.y += 1;
            if (stream.y.saturating_sub(stream.len)) > self.size.height {
                stream.alive = false;
            }
        }
    }

    pub fn run(&mut self) {
        let mut keys = async_stdin().keys();
        loop {
            self.streams.push(TextStream::new(
                self.rng.gen_range(0, self.size.width),
                self.rng.gen_range(4, 25),
            ));

            self.draw_streams();
            self.streams.retain(|stream| stream.alive);

            self.termbuf.draw().expect("error drawing terminal");
            match keys.next() {
                Some(Ok(Key::Char('q'))) | Some(Ok(Key::Ctrl('c'))) | Some(Ok(Key::Ctrl('d'))) => {
                    break
                }
                _ => {
                    thread::sleep(Duration::from_millis(self.opts.delay as u64));
                }
            }
        }
        // self.termbuf.set_cursor_visible(true).unwrap();
    }
}

fn main() -> Result<(), std::io::Error> {
    let opts = parse_args();
    let mut termbuf = termbuf::TermBuf::init()?;
    termbuf.set_cursor_visible(false)?;

    let size = termbuf.size()?;
    let rng = XorShiftRng::from_entropy();

    let mut app = App {
        termbuf,
        size,
        streams: vec![],
        opts,
        rng,
    };

    app.run();
    Ok(())
}

fn parse_args() -> Options {
    let matches = ClapApp::new("rMatrix")
        .version("2.0")
        .author("Noskcaj19")
        .about("The Matrix effect in your terminal")
        .arg(
            Arg::with_name("ascii")
                .short("a")
                .long("ascii")
                .help("Force ASCII only characters"),
        )
        .arg(
            Arg::with_name("no-bold")
                .short("n")
                .long("normal")
                .help("Use only normal weight characters"),
        )
        .arg(
            Arg::with_name("delay")
                .short("u")
                .long("delay")
                .help("Set the update delay")
                .default_value("45"),
        )
        .get_matches();

    let char_type = if matches.is_present("ascii") {
        CharType::Ascii
    } else {
        CharType::Kana
    };
    Options {
        char_type,
        delay: matches.value_of("delay").unwrap().parse().unwrap_or(45),
        bolding: !matches.is_present("no-bold"),
    }
}
