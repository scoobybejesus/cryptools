// Copyright (c) 2017-2023, scoobybejesus
// Redistributions must include the license: https://github.com/scoobybejesus/cryptools/blob/master/LEGAL.txt

use std::error::Error;

use ::ratatui::Terminal;
use ::ratatui::style::{Color, Modifier, Style};
use ::ratatui::text::{Text, Span, Line};
use ratatui::widgets::{Wrap, ListItem};
use ::ratatui::widgets::{Block, Borders, Paragraph, List};
use ::ratatui::layout::{Layout, Constraint, Direction};
use ::ratatui::backend::Backend;

use crate::mytui::app::{PrintWindow, REPORTS};


pub fn draw<B: Backend>(terminal: &mut Terminal<B>, app: &mut PrintWindow) -> Result<(), Box<dyn Error>> {

    terminal.draw(|f| {

        let instructions = vec![
            Line::from(vec![Span::raw("")]),

            Line::from(vec![
                Span::raw("  Press '"),
                Span::styled("x", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::raw("' to add the selected report to the list of reports to print/export."),
            ]),

            Line::from(vec![Span::raw("")]),
            
            Line::from(vec![
                Span::raw("  Press '"),
                Span::styled("d", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw("' to delete the selected report from the list of reports to print/export."),
            ]),
            
            Line::from(vec![Span::raw("")]),
            
            Line::from(vec![
                Span::raw("  Press '"),
                Span::styled("p", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::raw("' to print/export the selected reports."),
            ]),
            
            Line::from(vec![Span::raw("")]),
            
            Line::from(vec![
                Span::raw("  Press '"),
                Span::styled("q", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                Span::raw("' to quit without printing."),
            ]),
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
                Constraint::Length(instructions.len() as u16 + 2),
                Constraint::Length(REPORTS.len() as u16 + 2),
                Constraint::Length(rpts_to_prnt.len() as u16 + 2),
                Constraint::Length(1),
            ].as_ref())
            .split(f.size());

        let pg1 = Paragraph::new(instructions)
            .block(Block::default()
                .borders(Borders::NONE)
                .title(Span::styled(
                    "Instructions",
                    Style::default().fg(Color::LightMagenta).add_modifier(Modifier::BOLD).add_modifier(Modifier::UNDERLINED)
                ))
            )
            .wrap(Wrap {trim: false});
        f.render_widget(pg1, top_level_chunks[1]);

        let level_2_chunks = Layout::default()
            .constraints([Constraint::Percentage(10), Constraint::Percentage(80),Constraint::Percentage(10),].as_ref())
            .direction(Direction::Horizontal)
            .split(top_level_chunks[2]);

        let report_list_items: Vec<_> = app.tasks.items.iter().map(|i| ListItem::new(*i)).collect();

        let items = List::new(report_list_items)
            .block(Block::default()
                .borders(Borders::ALL)
                .title(Span::styled(
                    "Reports available for exporting",
                    Style::default().fg(Color::White).add_modifier(Modifier::BOLD).add_modifier(Modifier::UNDERLINED)
                ))
            )
            .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            .highlight_symbol(">");
        f.render_stateful_widget(items, level_2_chunks[1], &mut app.tasks.state);

        let level_2_chunks = Layout::default()
            .constraints([Constraint::Percentage(10), Constraint::Percentage(80),Constraint::Percentage(10),].as_ref())
            .direction(Direction::Horizontal)
            .split(top_level_chunks[3]);

        let rpts_to_prnt: Vec<_> = app.to_print_by_title.iter().map(|i| ListItem::new(*i)).collect();

        let to_print = List::new(rpts_to_prnt)
            .block(Block::default()
                .borders(Borders::ALL)
                .title(Span::styled(
                    "Reports to be exported",
                    Style::default().fg(Color::LightYellow).add_modifier(Modifier::BOLD).add_modifier(Modifier::UNDERLINED)
                ))
            );
        f.render_widget(to_print, level_2_chunks[1]);
    })?;

    Ok(())
}