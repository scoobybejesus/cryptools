// Copyright (c) 2017-2019, scoobybejesus
// Redistributions must include the license: https://github.com/scoobybejesus/cryptools/blob/master/LEGAL.txt

use std::io;

use ::tui::Terminal;
use ::tui::style::{Color, Modifier, Style};
use ::tui::widgets::{Widget, Block, Borders, SelectableList, Text, Paragraph};
use ::tui::layout::{Layout, Constraint, Direction};
use ::tui::backend::Backend;

use crate::mytui::app::{PrintWindow, REPORTS};


pub fn draw<B: Backend>(terminal: &mut Terminal<B>, app: &PrintWindow) -> Result<(), io::Error> {

    terminal.draw(|mut f| {

        let chunks = Layout::default()
            .constraints([
                Constraint::Length(1),
                Constraint::Length(8),
                Constraint::Length(REPORTS.len() as u16 + 2),
                Constraint::Percentage(35)
            ].as_ref())
            .split(f.size());

        let text = [
            Text::raw("\nPress '"),
            Text::styled("x", Style::default().fg(Color::LightGreen)),
            Text::raw("' to add the selected report to the list of reports to print/export.\n"),
            Text::raw("\nPress '"),
            Text::styled("p", Style::default().fg(Color::Green)),
            Text::raw("' to print/export the selected reports.\n"),
            Text::raw("\nPress '"),
            Text::styled("q", Style::default().fg(Color::Red)),
            Text::raw("' to quit without printing.\n\n"),
        ];

        Paragraph::new(text.iter())
            .block(
                Block::default()
                    .borders(Borders::NONE)
                    .title("Instructions")
                    .title_style(Style::default().fg(Color::Blue).modifier(Modifier::BOLD)),
            )
            .wrap(true)
            .render(&mut f, chunks[1]);

        let draw_chunk = Layout::default()
            .constraints([Constraint::Percentage(10), Constraint::Percentage(80),Constraint::Percentage(10),].as_ref())
            .direction(Direction::Horizontal)
            .split(chunks[2]);

        SelectableList::default()
            .block(Block::default().borders(Borders::ALL).title("Report List"))
            .items(&app.tasks.items)
            .select(Some(app.tasks.selected))
            .highlight_style(Style::default().fg(Color::Yellow).modifier(Modifier::BOLD))
            .highlight_symbol(">")
            .render(&mut f, draw_chunk[1]);

    })
}