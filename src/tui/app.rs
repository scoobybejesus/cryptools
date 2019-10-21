// Copyright (c) 2017-2019, scoobybejesus
// Redistributions must include the license: https://github.com/scoobybejesus/cryptools/blob/master/LEGAL.txt


pub (crate) const REPORTS: [&'static str; 9] = [
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
            tasks: ListState::new(REPORTS.to_vec()),
            to_print: Vec::with_capacity(REPORTS.len() + 3),
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
                self.to_print = Vec::with_capacity(0);
                self.should_quit = true;
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