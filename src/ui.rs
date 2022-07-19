use std::borrow::BorrowMut;

use tui::{
	backend::{Backend},
	layout::{Constraint, Direction, Layout},
	style::{Color, Modifier, Style},
	widgets::{Block, Borders, Paragraph, Cell, Row, Table},
	Frame,
};
use unicode_width::UnicodeWidthStr;

use crate::state::State;

const SPINNER_FRAMES: [char; 8] = ['⣾','⣽','⣻','⢿','⡿','⣟','⣯','⣷'];

pub fn ui<B: Backend>(f: &mut Frame<B>, state: &std::cell::RefCell<State>) {
	let areas = Layout::default()
		.direction(Direction::Vertical)
		.constraints([
			Constraint::Length(3),
			Constraint::Min(0),
		].as_ref())
		.split(f.size());

	{ // input

		let mut title = "Search".to_string();

		if state.borrow().pending_searches > 0 {
			let idx = state.borrow().spinner_index;
			title.push(' ');
			title.push(SPINNER_FRAMES[idx]);
			state.borrow_mut().spinner_index = (idx + 1) % SPINNER_FRAMES.len();
		} else if state.borrow().spinner_index > 0 {
			state.borrow_mut().spinner_index = 0;
		}

		let input_block = Block::default()
			.borders(Borders::ALL)
			.title(title)
			.border_style(Style::default().fg(Color::Yellow));

		let input = Paragraph::new(state.borrow().input.to_string())
			.block(input_block);

		f.render_widget(input, areas[0]);
		f.set_cursor(areas[0].x + (state.borrow().input.width() as u16) + 1, areas[0].y + 1);
	}
	{ // table
    let selected_style = Style::default().add_modifier(Modifier::REVERSED);
    let normal_style = Style::default().bg(Color::Blue);
    let header_cells = [
			"Name",
			"Description",
			"Popularity",
			"Last Modified",
		]
			.iter()
			.map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow)));
		let header = Row::new(header_cells)
			.style(normal_style)
			.height(1)
			.bottom_margin(1);
		let rows: Vec<Row> = state.borrow().results.items.iter().map(|item| {
			Row::new(vec![
				Cell::from(item.name.clone()),
				Cell::from(item.description.clone().unwrap_or_default()),
				Cell::from(item.popularity.clone().to_string()),
				Cell::from(item.last_modified.to_string()),
			])
		}).collect();
		let table = Table::new(rows)
			.header(header)
			.block(Block::default().borders(Borders::ALL).title("Packages"))
			.highlight_style(selected_style)
			.widths(&[
				Constraint::Percentage(20),
				Constraint::Percentage(65),
				Constraint::Percentage(5),
				Constraint::Percentage(10),
			]);
		f.render_stateful_widget(table, areas[1], &mut state.borrow_mut().results.state);
	}

}