extern crate rand;
extern crate pancurses;
extern crate terminal_size;

use std::cell::RefCell;
use rand::{thread_rng, Rng};
use pancurses::{initscr, endwin, noecho, curs_set, Input};
use terminal_size::{Width, Height, terminal_size};

fn rand_char() -> char {
	thread_rng().gen_range(b'!', b'}') as char
}

struct TextStream {
	x: usize,
	y: RefCell<usize>,
	len: usize,
}

impl TextStream {
	fn new(len: usize, x: usize) -> TextStream {
		TextStream {
			x,
			y: RefCell::new(0),
			len
		}
	}
	fn make_stream(width: usize, height: usize) -> TextStream {
		let x_pos = rand::thread_rng().gen_range(0, width);
		let len = rand::thread_rng().gen_range(0, height);
		TextStream::new(len, x_pos)
	}
	fn draw(&self, window: &pancurses::Window, height: usize) -> bool {
		let y = *self.y.borrow() as i32;
		let x = self.x as i32;
		// Add char
		window.mvaddch(y, x, rand_char());
		// Set leader color to white
		window.mv(y, x);
		window.chgat(1, 0, 2);
		// Add random bolding
		window.mv(y-1, x);
		if thread_rng().gen::<u8>() < 60u8 /* .3 */ {
			window.chgat(1, pancurses::A_BOLD, 1);
		} else {
			window.chgat(1, 0, 1);
		}
		let y = y as usize;
		if y >= self.len {
			let last = (y - self.len) as i32;
			window.mvaddch(last, x, ' ');
			if (y - self.len) > height {
				return false
			}
		}
		*self.y.borrow_mut() += 1;
		true
	}
}

fn main() {
	let _ = get_size();
	let window = initscr();
	window.nodelay(true);
	window.refresh();
	noecho();
	if pancurses::has_colors() {
		pancurses::start_color();
		pancurses::use_default_colors();
	}
	pancurses::init_pair(1, pancurses::COLOR_GREEN, -1);
	pancurses::init_pair(2, pancurses::COLOR_WHITE, -1);
	window.attrset(pancurses::COLOR_PAIR(1));
	curs_set(0);
	let mut streams = Vec::new();
	loop {
		let (width, height) = get_size();
		for _ in 0..4 {
			if let Some(new_stream) = new_stream(width, height, &streams) {
				streams.push(new_stream);
			}
		}
		streams.retain(|ref stream| {
			stream.draw(&window, height)
		});
		window.refresh();
		match window.getch() {
			Some(Input::Character('q')) => break,
			_ => (),
		}
		std::thread::sleep(std::time::Duration::from_millis(100));
	}
	endwin();
}

fn check_stream(new_stream: &TextStream, height: usize, streams: &Vec<TextStream>) -> bool {
	for stream in streams {
		if stream.x == new_stream.x {
			let stream_end = if stream.x > stream.len {stream.x - stream.len} else {stream.x};
			if stream_end < height/3 {
				return false
			}
		}
	}
	return true
}

fn new_stream(width: usize, height: usize, streams: &Vec<TextStream>) -> Option<TextStream> {
	let new_stream = TextStream::make_stream(width, height);
	if check_stream(&new_stream, height, streams) {
		Some(new_stream)
	} else {
		None
	}
}

fn get_size() -> (usize, usize) {
	if let Some((w, h)) = term_size() {
		return (w as usize, h as usize);
	} else {
		println!("Unable to get console size");
		std::process::exit(1);
	}
}

fn term_size() -> Option<(u16, u16)> {
	let size = terminal_size();
	if let Some((Width(w), Height(h))) = size {
		return Some((w, h))
	} else {
		return None
	}
}