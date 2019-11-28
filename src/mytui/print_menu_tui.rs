// Copyright (c) 2017-2019, scoobybejesus
// Redistributions must include the license: https://github.com/scoobybejesus/cryptools/blob/master/LEGAL.txt

use std::io;
use std::time::Duration;
use std::collections::{HashMap};
use std::error::Error;

use tui::Terminal;
use tui::backend::TermionBackend;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use termion::input::MouseTerminal;
use termion::event::Key;

use crptls::transaction::{Transaction, ActionRecord};
use crptls::account::{Account, RawAccount};
use crptls::core_functions::{ImportProcessParameters};

use crate::mytui::event::{Events, Event, Config};
use crate::mytui::ui as ui;
use crate::mytui::app as app;


pub (crate) fn print_menu_tui(
    settings: &ImportProcessParameters,
    action_records_map: &HashMap<u32, ActionRecord>,
    raw_acct_map: &HashMap<u16, RawAccount>,
    account_map: &HashMap<u16, Account>,
    transactions_map: &HashMap<u32, Transaction>,
) -> Result<(), Box<dyn Error>> {

    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    let mut app = app::PrintWindow::new("Reports");

    let events = Events::with_config(Config {
        tick_rate: Duration::from_millis(250u64),
        ..Config::default()
    });

    loop {

        ui::draw(&mut terminal, &app)?;

        if let Event::Input(key) = events.next()? {

            match key {

                Key::Char(c) => {
                    app.on_key(c);
                }
                Key::Up => {
                    app.on_up();
                }
                Key::Down => {
                    app.on_down();
                }
                _ => {}
            }
        }

        if app.should_quit {
            break;
        }
    }

    // Seem to need both of these for the native terminal to be available for println!()'s below
    std::mem::drop(terminal);
    std::thread::sleep(Duration::from_millis(10));

    app::export(
        &app,
        &settings,
        &action_records_map,
        &raw_acct_map,
        &account_map,
        &transactions_map
    )?;

    Ok(())
}