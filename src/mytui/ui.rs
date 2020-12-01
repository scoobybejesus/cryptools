// Copyright (c) 2017-2020, scoobybejesus
// Redistributions must include the license: https://github.com/scoobybejesus/cryptools/blob/master/LEGAL.txt

use std::io;

use ::tui::Terminal;
use ::tui::style::{Color, Modifier, Style};
use ::tui::widgets::{Widget, Block, Borders, SelectableList, Text, Paragraph, List};
use ::tui::layout::{Layout, Constraint, Direction};
use ::tui::backend::Backend;

use crate::mytui::app::{PrintWindow, REPORTS};


pub fn draw<B: Backend>(terminal: &mut Terminal<B>, app: &PrintWindow) -> Result<(), io::Error> {

    terminal.draw(|mut f| {

        let instructions = [
            Text::raw("\nPress '"),
            Text::styled("x", Style::default().fg(Color::Cyan).modifier(Modifier::BOLD)),
            Text::raw("' to add the selected report to the list of reports to print/export.\n"),
            Text::raw("\nPress '"),
            Text::styled("d", Style::default().fg(Color::Yellow).modifier(Modifier::BOLD)),
            Text::raw("' to delete the selected report from the list of reports to print/export.\n"),
            Text::raw("\nPress '"),
            Text::styled("p", Style::default().fg(Color::Green).modifier(Modifier::BOLD)),
            Text::raw("' to print/export the selected reports.\n"),
            Text::raw("\nPress '"),
            Text::styled("q", Style::default().fg(Color::Red).modifier(Modifier::BOLD)),
            Text::raw("' to quit without printing.\n\n"),
        ];
        let rpts_to_prnt = app.to_print_by_title.iter().map(|&rpt_to_prnt| {
            Text::styled(
                format!("{}", rpt_to_prnt),
                Style::default().fg(Color::White)
            )
        });

        let top_level_chunks = Layout::default()
            .constraints([
                Constraint::Length(1),
                Constraint::Length(2 + (instructions.len() as u16 / 3 * 2)), // 2 for title "Instructions", plus 3 TEXT array elements/instruction
                Constraint::Length(REPORTS.len() as u16 + 2),
                Constraint::Length(rpts_to_prnt.len() as u16 + 2),
                Constraint::Length(1),
            ].as_ref())
            .split(f.size());

        Paragraph::new(instructions.iter())
            .block(
                Block::default()
                    .borders(Borders::NONE)
                    .title("Instructions")
                    .title_style(Style::default().fg(Color::Blue).modifier(Modifier::BOLD).modifier(Modifier::UNDERLINED)),
            )
            .wrap(true)
            .render(&mut f, top_level_chunks[1]);

        let level_2_chunks = Layout::default()
            .constraints([Constraint::Percentage(10), Constraint::Percentage(80),Constraint::Percentage(10),].as_ref())
            .direction(Direction::Horizontal)
            .split(top_level_chunks[2]);

        SelectableList::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Reports available for exporting")
                    .title_style(Style::default().fg(Color::White).modifier(Modifier::BOLD).modifier(Modifier::UNDERLINED))
            )
            .items(&app.tasks.items)
            .select(Some(app.tasks.selected))
            .highlight_style(Style::default().fg(Color::Yellow).modifier(Modifier::BOLD))
            .highlight_symbol(">")
            .render(&mut f, level_2_chunks[1]);

        let level_2_chunks = Layout::default()
            .constraints([Constraint::Percentage(10), Constraint::Percentage(80),Constraint::Percentage(10),].as_ref())
            .direction(Direction::Horizontal)
            .split(top_level_chunks[3]);

        List::new(rpts_to_prnt)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Reports to be exported")
                    .title_style(Style::default().fg(Color::LightYellow).modifier(Modifier::BOLD).modifier(Modifier::UNDERLINED))
            )
            .render(&mut f, level_2_chunks[1]);
    })
}