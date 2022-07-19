

use crate::search;

use tui::{
	widgets::{TableState},
};

pub struct StatefulTable<T> {
	pub state: TableState,
	pub items: Vec<T>,
}

impl<T> StatefulTable<T> {
	#[must_use] pub fn new(items: Vec<T>) -> StatefulTable<T> {
		StatefulTable {
			state: TableState::default(),
			items,
		}
	}

	pub fn next(&mut self) {
		if self.items.is_empty() { return; };

		self.state.select(Some(match self.state.selected() {
			Some(i) => if i == self.items.len() - 1 { 0 } else { (self.items.len() - 1).min(i + 1) },
			None => 0,
		}));
	}

	pub fn previous(&mut self) {
		if self.items.is_empty() { return; };

		self.state.select(Some(match self.state.selected() {
			Some(i) => if i == 0 { self.items.len() - 1 } else { 0.max(i - 1) },
			None => self.items.len() - 1,
		}));
	}
}


pub struct State {
	pub input: String,
	pub results: StatefulTable<search::Package>,
	pub pending_searches: usize,
	pub spinner_index: usize,
}

impl Default for State {
	fn default() -> State {
		State {
			input: String::new(),
			results: StatefulTable::new(Vec::new()),
			pending_searches: 0,
			spinner_index: 0,
		}
	}
}
