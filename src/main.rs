use crossterm::{
	event,
	execute,
	terminal,
};
use futures::{StreamExt, FutureExt};

mod search;
mod state;
mod ui;

use state::State;
use ui::ui;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	start().await;
	Ok(())
}

async fn start() {
	let (tx, rx) = tokio::sync::mpsc::channel::<String>(10);
	// create state and run it
	let state = std::cell::RefCell::new(State::default());

	tokio::join!(
		ui_run(&state, tx),
		search_run(&state, rx),
	);
}

async fn search_run(state: &std::cell::RefCell<State>, mut rx_input: tokio::sync::mpsc::Receiver<String>) {
	let mut cache = std::collections::HashMap::<String, Vec<search::Package>>::new();
	while let Some(input) = rx_input.recv().await {
		if !state.borrow().input.eq(&input) { continue; }

		if !cache.contains_key(&input) {
			state.borrow_mut().pending_searches += 1;
			let results = search::search(search::By::NameDesc, input.as_str()).await;
			state.borrow_mut().pending_searches -= 1;
			cache.insert(input.clone(), results);
		}

		if state.borrow().input.eq(&input) {
			if let Some(results) = cache.get(&input) {
				state.borrow_mut().results = state::StatefulTable::new(results.clone());
			}
		}
	}
}

async fn ui_run(state: &std::cell::RefCell<State>, tx_input: tokio::sync::mpsc::Sender<String>) {
	let stop = std::cell::Cell::<bool>::new(false);
	tokio::join!(
		async {
			terminal::enable_raw_mode().unwrap();
			let mut stdout = std::io::stdout();
			execute!(stdout, terminal::EnterAlternateScreen, event::EnableMouseCapture).unwrap();
			let backend = tui::backend::CrosstermBackend::new(stdout);
			let mut terminal = tui::Terminal::new(backend).unwrap();

			loop {
				if stop.get() {
					break;
				}
				if terminal.draw(|f| ui(f, state)).is_err() {
					break;
				}
				tokio::time::sleep(std::time::Duration::new(0, 0)).await;
			}

			// restore terminal
			terminal::disable_raw_mode().unwrap();
			execute!(
				terminal.backend_mut(),
				terminal::LeaveAlternateScreen,
				event::DisableMouseCapture
			).unwrap();
			terminal.show_cursor().unwrap();
		},
		async {
			let mut reader = crossterm::event::EventStream::new();

			loop {
				if let Some(Ok(event::Event::Key(key))) = reader.next().fuse().await {
					match key.code {
						event::KeyCode::Esc => {
							stop.set(true);
							break;
						}
						event::KeyCode::Char(c) => {
							if c == 'c' && key.modifiers == event::KeyModifiers::CONTROL {
								stop.set(true);
								break;
							}

							state.borrow_mut().input.push(c);
							let input = state.borrow().input.clone();
							tx_input.send(input).await.unwrap();
						}
						event::KeyCode::Backspace => {
							state.borrow_mut().input.pop();
							let input = state.borrow().input.clone();
							tx_input.send(input).await.unwrap();
						},
						event::KeyCode::Up => { state.borrow_mut().results.previous(); },
						event::KeyCode::Down => { state.borrow_mut().results.next(); },
						_ => {}
					}
				}
			}
		}
	);
}