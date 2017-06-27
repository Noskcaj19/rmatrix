extern crate rand;
extern crate pancurses;
extern crate terminal_size;

use std::cell::RefCell;
use rand::{thread_rng, Rng};
use pancurses::{initscr, endwin, noecho, curs_set};
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
	fn draw(&self, window: &pancurses::Window, height: usize) -> bool {
		window.mvaddch(*self.y.borrow() as i32, self.x as i32, rand_char());
		let y = *self.y.borrow();
		if y >= self.len {
			let last = (y - self.len) as i32;
			window.mvaddch(last, self.x as i32, ' ');	
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
	window.refresh();
	noecho();
	if pancurses::has_colors() {
		pancurses::start_color();
	}
	pancurses::init_pair(1, pancurses::COLOR_GREEN, pancurses::COLOR_BLACK);
	window.attrset(pancurses::COLOR_PAIR(1));
	curs_set(0);
	let mut streams = Vec::new();
	loop {
		let (width, height) = get_size();
		let new_stream = make_stream(width, height);
		streams.push(new_stream);
		let new_stream2 = make_stream(width, height);
		streams.push(new_stream2);
		streams.retain(|ref stream| {
			stream.draw(&window, height)
		});
		window.refresh();
		std::thread::sleep(std::time::Duration::from_millis(100));
	}
	endwin();
}

fn make_stream(width: usize, height: usize) -> TextStream {
	let x_pos = rand::thread_rng().gen_range(0, width);
	let len = rand::thread_rng().gen_range(0, height);
	TextStream::new(len, x_pos)
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