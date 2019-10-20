// Copyright (c) 2017-2019, scoobybejesus
// Redistributions must include the license: https://github.com/scoobybejesus/cryptools/blob/master/LEGAL.txt

#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_assignments)]
// Note: the above are possibly temporary, to silence "x was not used" warnings.
// #[warn(dead_code)] is the default (same for unused_variables)


use std::ffi::OsString;
use std::path::PathBuf;
use std::error::Error;
use std::io;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use tui::Terminal;
use tui::backend::{TermionBackend, Backend};
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Widget, Block, Borders, SelectableList, Text, Paragraph};
use tui::layout::{Layout, Constraint, Direction};
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use termion::input::MouseTerminal;
use termion::event::Key;
use termion::input::TermRead;
use structopt::StructOpt;

mod account;
mod transaction;
mod core_functions;
mod csv_import_accts_txns;
mod create_lots_mvmts;
mod import_cost_proceeds_etc;
mod cli_user_choices;
mod csv_export;
mod txt_export;
mod string_utils;
mod decimal_utils;
mod tests;
mod wizard;
mod skip_wizard;
mod setup;


#[derive(StructOpt, Debug)]
#[structopt(name = "cryptools")]
pub(crate) struct Cli {

    #[structopt(flatten)]
    flags: Flags,

    #[structopt(flatten)]
    opts: Options,

    /// File to be imported.  (Currently, the only supported date format is %m/%d/%y.)
    #[structopt(name = "file", parse(from_os_str))]
    file_to_import: Option<PathBuf>,
}

#[derive(StructOpt, Debug)]
pub(crate) struct Flags {

    /// User is instructing the program to skip the data entry wizard.
    /// When set, program will error without required command-line args.
    #[structopt(name = "accept args", short = "a", long = "accept")]
    accept_args: bool,

    /// This will cause the program to expect the txDate field in the file_to_import to use the format
    /// YYYY-MM-dd or YY-MM-dd (or YYYY/MM/dd or YY/MM/dd, depending on the date-separator option)
    /// instead of the default US-style MM-dd-YYYY or MM-dd-YY (or MM/dd/YYYY or MM/dd/YY, depending on the
    /// date separator option).
    #[structopt(name = "date conforms to ISO 8601", short = "i", long = "iso")]
    iso_date: bool,

    /// Once the import file has been fully processed, the user will be presented
    /// with a menu for manually selecting which reports to print/export.
    #[structopt(name = "print menu", short, long = "print-menu")]
    print_menu: bool,

    /// This will prevent the program from writing reports to files.
    /// This will be ignored if -a is not set (the wizard will always ask to output).
    #[structopt(name = "suppress reports", short, long = "suppress")]
    suppress_reports: bool,
}

#[derive(StructOpt, Debug)]
pub(crate) struct Options {

    /// Choose "h", "s", or "p" for hyphen, slash, or period (i.e., "-", "/", or ".") to indicate the separator
    /// character used in the file_to_import txDate column (i.e. 2017/12/31, 2017-12-31, or 2017.12.31).
    #[structopt(name = "date separator character", short, long = "date-separator", default_value = "h", parse(from_os_str))]
    date_separator: OsString,

    /// Home currency (currency in which all resulting reports are denominated).
    /// (Only available as a command line setting.)
    #[structopt(name = "home currency", short = "c", long = "currency", default_value = "USD", parse(from_os_str))]
    home_currency: OsString,

    /// Cutoff date through which like-kind exchange treatment should be applied.
    /// Please use %y-%m-%d (or %Y-%m-%d) format for like-kind cutoff date entry.
    #[structopt(name = "like-kind cutoff date", short, long = "lk-cutoff", parse(from_os_str))]
    lk_cutoff_date: Option<OsString>,

    /// Inventory costing method (in terms of lot selection, i.e., LIFO, FIFO, etc.).
    /// There are currently four options (1 through 4).
    #[structopt(name = "method number for lot selection", short, long, default_value = "1", parse(from_os_str), long_help =
    r"    1. LIFO according to the order the lot was created.
    2. LIFO according to the basis date of the lot.
    3. FIFO according to the order the lot was created.
    4. FIFO according to the basis date of the lot.
    ")]
    inv_costing_method: OsString,

    /// Output directory for exported reports.
    #[structopt(name = "output directory", short, long = "output", default_value = ".", parse(from_os_str))]
    output_dir_path: PathBuf,
}



fn main() -> Result<(), Box<dyn Error>> {

    let args = Cli::from_args();

    println!(
    "
    Hello, crypto-folk!  Welcome to cryptools!

    This software will import your csv file's ledger of cryptocurrency transactions.
    It will then process it by creating 'lots' and posting 'movements' to those lots.
    Along the way, it will keep track of income, expenses, gains, and losses.

    Note: it is designed to import a full history. Gains and losses may be incorrect otherwise.
    ");

    let (input_file_path, settings) = setup::run_setup(args)?;

    let (
        account_map,
        raw_acct_map,
        action_records_map,
        transactions_map,
    ) = core_functions::import_and_process_final(input_file_path, &settings)?;

    let mut should_export_all = settings.should_export;
    let present_print_menu_tui = settings.print_menu;

    if present_print_menu_tui { should_export_all = false }

    if should_export_all {

        println!("Creating reports now.");

        csv_export::_1_account_sums_to_csv(
            &settings,
            &raw_acct_map,
            &account_map
        );

        csv_export::_2_account_sums_nonzero_to_csv(
            &account_map,
            &settings,
            &raw_acct_map
        );

        csv_export::_3_account_sums_to_csv_with_orig_basis(
            &settings,
            &raw_acct_map,
            &account_map
        );

        csv_export::_4_transaction_mvmt_detail_to_csv(
            &settings,
            &action_records_map,
            &raw_acct_map,
            &account_map,
            &transactions_map
        )?;

        csv_export::_5_transaction_mvmt_summaries_to_csv(
            &settings,
            &action_records_map,
            &raw_acct_map,
            &account_map,
            &transactions_map
        )?;

        csv_export::_6_transaction_mvmt_detail_to_csv_w_orig(
            &settings,
            &action_records_map,
            &raw_acct_map,
            &account_map,
            &transactions_map
        )?;

        txt_export::_1_account_lot_detail_to_txt(
            &settings,
            &raw_acct_map,
            &account_map,
            &transactions_map,
            &action_records_map
        )?;

        txt_export::_2_account_lot_summary_to_txt(
            &settings,
            &raw_acct_map,
            &account_map,
        )?;

        txt_export::_3_account_lot_summary_non_zero_to_txt(
            &settings,
            &raw_acct_map,
            &account_map,
        )?;

    }

    if present_print_menu_tui {

        const TASKS: [&'static str; 9] = [
            "1. CSV: Account Sums",
            "2. CSV: Account Sums (Non-zero only)",
            "3. CSV: Account Sums (Orig. basis vs like-kind basis)",
            "4. CSV: Transactions by movement (every movement)",
            "5. CSV: Transactions by movement (summarized by long-term/short-term)",
            "6. CSV: Transactions by movement (every movement, w/ orig. and like-kind basis",
            "7. TXT: Accounts by lot (every movement)",
            "8. TXT: Accounts by lot (every lot balance)",
            "9. TXT: Accounts by lot (every non-zero lot balance)",
        ];

        pub struct ListState<I> {
            pub items: Vec<I>,
            pub selected: usize,
        }

        impl<I> ListState<I> {
            fn new(items: Vec<I>) -> ListState<I> {
                ListState { items, selected: 0 }
            }
            fn select_previous(&mut self) {
                if self.selected > 0 {
                    self.selected -= 1;
                }
            }
            fn select_next(&mut self) {
                if self.selected < self.items.len() - 1 {
                    self.selected += 1
                }
            }
        }

        pub struct PrintWindow<'a> {
            pub title: &'a str,
            pub should_quit: bool,
            pub tasks: ListState<(&'a str)>,
            pub to_print: Vec<usize>,
        }

        impl<'a> PrintWindow<'a> {
            pub fn new(title: &'a str) -> PrintWindow<'a> {
                PrintWindow {
                    title,
                    should_quit: false,
                    tasks: ListState::new(TASKS.to_vec()),
                    to_print: Vec::with_capacity(TASKS.len() + 3),
                }
            }

            pub fn on_up(&mut self) {
                self.tasks.select_previous();
            }

            pub fn on_down(&mut self) {
                self.tasks.select_next();
            }

            pub fn on_key(&mut self, c: char) {
                match c {
                    'q' => {
                        self.should_quit = true;
                        self.to_print = Vec::with_capacity(0)
                    }
                    'p' => {
                        Self::change_vec_to_chrono_order_and_dedup(&mut self.to_print);
                        self.should_quit = true;
                    }
                    'x' => {
                        self.to_print.push(self.tasks.selected)
                    }
                    _ => {}
                }
            }
            fn change_vec_to_chrono_order_and_dedup(vec: &mut Vec<usize>) {
                let length = vec.len();
                for _ in 0..length {
                    for j in 0..length-1 {
                        if vec[j] > vec[j+1] {
                            vec.swap(j, j+1)
                        }

                    }
                }
                vec.dedup();
            }
        }

        let stdout = io::stdout().into_raw_mode()?;
        let stdout = MouseTerminal::from(stdout);
        let stdout = AlternateScreen::from(stdout);
        let backend = TermionBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        terminal.hide_cursor()?;

        pub fn draw<B: Backend>(terminal: &mut Terminal<B>, app: &PrintWindow) -> Result<(), io::Error> {
            terminal.draw(|mut f| {
                let chunks = Layout::default()
                    .constraints([
                        Constraint::Length(1),
                        Constraint::Length(8),
                        Constraint::Length(TASKS.len() as u16 + 2),
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


        pub enum Event<I> {
            Input(I),
            Tick,
        }
        pub struct Events {
            rx: mpsc::Receiver<Event<Key>>,
            input_handle: thread::JoinHandle<()>,
            tick_handle: thread::JoinHandle<()>,
        }
        let events = Events::with_config(Config {
            tick_rate: Duration::from_millis(250u64),
            ..Config::default()
        });
        #[derive(Debug, Clone, Copy)]
        pub struct Config {
            pub exit_key: Key,
            pub tick_rate: Duration,
        }

        impl Default for Config {
            fn default() -> Config {
                Config {
                    exit_key: Key::Char('q'),
                    tick_rate: Duration::from_millis(250),
                }
            }
        }

        impl Events {
            pub fn new() -> Events {
                Events::with_config(Config::default())
            }

            pub fn with_config(config: Config) -> Events {
                let (tx, rx) = mpsc::channel();
                let input_handle = {
                    let tx = tx.clone();
                    thread::spawn(move || {
                        let stdin = io::stdin();
                        for evt in stdin.keys() {
                            match evt {
                                Ok(key) => {
                                    if let Err(_) = tx.send(Event::Input(key)) {
                                        return;
                                    }
                                    if key == config.exit_key {
                                        return;
                                    }
                                }
                                Err(_) => {}
                            }
                        }
                    })
                };
                let tick_handle = {
                    let tx = tx.clone();
                    thread::spawn(move || {
                        let tx = tx.clone();
                        loop {
                            tx.send(Event::Tick).unwrap();
                            thread::sleep(config.tick_rate);
                        }
                    })
                };
                Events {
                    rx,
                    input_handle,
                    tick_handle,
                }
            }

            pub fn next(&self) -> Result<Event<Key>, mpsc::RecvError> {
                self.rx.recv()
            }
        }

        let mut app = PrintWindow::new("Reports");
        loop {
            draw(&mut terminal, &app)?;
            match events.next()? {
                Event::Input(key) => match key {
                    Key::Char(c) => {
                        app.on_key(c);
                    }
                    Key::Up => {
                        app.on_up();
                    }
                    Key::Down => {
                        app.on_down();
                    }
                    Key::Left => {
                        // app.on_left();
                    }
                    Key::Right => {
                        // app.on_right();
                    }
                    _ => {}
                },
                _ => {}
            }
            if app.should_quit {
                break;
            }
        }

        // Seem to need both of these for the native terminal to be available for println!()'s below
        std::mem::drop(terminal);
        std::thread::sleep(Duration::from_millis(10));

        for report in app.to_print {
            println!("Exporting: {}", TASKS[report]);
            match report + 1 {
                1 => {
                    csv_export::_1_account_sums_to_csv(
                        &settings,
                        &raw_acct_map,
                        &account_map
                    );
                }
                2 => {
                    csv_export::_2_account_sums_nonzero_to_csv(
                        &account_map,
                        &settings,
                        &raw_acct_map
                    );
                }
                3 => {
                    csv_export::_3_account_sums_to_csv_with_orig_basis(
                        &settings,
                        &raw_acct_map,
                        &account_map
                    );
                }
                4 => {
                    csv_export::_4_transaction_mvmt_detail_to_csv(
                        &settings,
                        &action_records_map,
                        &raw_acct_map,
                        &account_map,
                        &transactions_map
                    )?;
                }
                5 => {
                    csv_export::_5_transaction_mvmt_summaries_to_csv(
                        &settings,
                        &action_records_map,
                        &raw_acct_map,
                        &account_map,
                        &transactions_map
                    )?;
                }
                6 => {
                    csv_export::_6_transaction_mvmt_detail_to_csv_w_orig(
                        &settings,
                        &action_records_map,
                        &raw_acct_map,
                        &account_map,
                        &transactions_map
                    )?;
                }
                7 => {
                    txt_export::_1_account_lot_detail_to_txt(
                        &settings,
                        &raw_acct_map,
                        &account_map,
                        &transactions_map,
                        &action_records_map
                    )?;
                }
                8 => {
                    txt_export::_2_account_lot_summary_to_txt(
                        &settings,
                        &raw_acct_map,
                        &account_map,
                    )?;
                }
                9 => {
                    txt_export::_3_account_lot_summary_non_zero_to_txt(
                        &settings,
                        &raw_acct_map,
                        &account_map,
                    )?;
                }
                _ => {}
            }
        }
    }

    // use tests::test;
    // test::run_tests(
    //     &transactions_map,
    //     &action_records_map,
    //     &account_map
    // );


    Ok(())

}
