# cryptools

### Accounting library for cryptocurrency transaction activity.

###### (The package produces a binary and accompanying library)

This is a command-line tool that calculates income, expenses, realized gains, realized losses,
and holding period from cryptocurrency activity and denominates the results in the user's home currency.
The default home currency is USD, but any currency can be substituted.
This tool is probably most useful for filling out a tax return or making tax planning decisions.
It is already mildly difficult to do the prep work (CSV input file, below) for using a tool like this,
so a person wanted this for a quick fix may be disappointed.

---

Given a [CSV input file](https://github.com/scoobybejesus/cryptools/blob/master/InputFile_CSV.md)
containing the user's entire cryptocurrency transaction history, the software will:

* record every cryptocurrency acquisition and track* it until it is disposed
* assign cost basis to every acquisition as of the date of purchase/exchange/receipt
* track the original acquisition date and cost basis (making adjustements for like-kind exchange treatment, if elected)
* compute gain or loss from the sale/exchange/disposal (including whether short-term or long-term)
* record income for incoming transactions and expenses for outgoing transactions
* print/export the results as CSV and TXT files

*The tracking isn't pooled by `ticker`.  Rather, it's tracked at the account/wallet level.

There is a helper Python script at the root of the repo that will assist you in sanitizing your CSV file
so it can be successfully imported into `cryptools`.

---

### Features

* Two methods each of LIFO or FIFO (compatible w/ the concept of "specific identification")

* Ability to perform like-kind exchange treatment through a particular date

* Compatible with any (single) home currency

* Will export all bookkeeping journal entries (w/ `-a` or `-j`)

* Print menu (via `-p`) for individually choosing the desired reports

### Constraints

* *All* cryptocurrency-related activity for the user generally must be included in the
[CSV input file](https://github.com/scoobybejesus/cryptools/blob/master/InputFile_CSV.md).

* There can only be either one or two accounts used in a given transaction
(i.e., if a Counterparty token or Ethereum token transaction must be recorded,
the BTC or ETH transaction fee must be reflected in a separate transaction row).

* Manual adjustments may need to be made to the output files in cases, for example,
when appreciated cryptocurrency was used to make a tax-deductible charitable contribution.

* Precision is limited to eight decimal places.  Additional digits will be stripped during
import and may cause unintended rounding issues.

* Microsoft Excel.  Don't let Excel cause you to bang your head against a wall.
Picture this scenario.  You keep your transactions for your input file in a Google Sheet,
and you're meticulous about making sure it's perfect.
You then download it as a CSV file and import it into `cryptools`.
It works perfectly, and you have all your reports.
Then you realize you'd like to quickly change a memo and re-run the reports, so you open the CSV file in Excel and edit it.
Then you import it into `cryptools` again and the program panics!
What happened is most likely that Excel changed the rounding of your precise decimals underneath you!
Depending on the rounding, `cryptools` may think your input file has been incorrectly prepared
because you've supposedly spent more coins than you actually owned at that time.
`Cryptools` does not let you spend coins you don't own, and it will exit upon finding such a condition.
The program is right, and your data is right, but Excel modified your data, so the program crashed for "no reason."
The solution is to have Excel already open, then in the ribbon's Data tab, you'll import your CSV file "From Text."
You'll choose Delimited, and Comma, and then highlight every column and choose Text as the data type.

* Currently, does not build on Windows due to the Termion crate (used for the print menu).

## Installation

1. `git clone https://github.com/scoobybejesus/cryptools.git`
2. `cd cryptools`
3. `cargo build` (include `--release` for a non-debug build)

This will build `./target/debug/cryptools` (or `./target/release/cryptools` for a non-debug build).

### Note on Windows

Windows won't build with the current TUI print menu.  To build on Windows, try with `cargo build --no-default-features`.

## Usage

Run `./target/debug/cryptools` with no arguments (or with `--help`, or `-h`) to see usage.
Alternatively, run `cargo run`, in which case command-line parameters for `cryptools` may be entered following `--`,
e.g., `cargo run -- -h` or `cargo run -- my_input_file.csv -ai`.

Running with no options/arguments will lead the user through a wizard.
To skip the wizard, there are three requirements:
* The [CSV input file](https://github.com/scoobybejesus/cryptools/blob/master/InputFile_CSV.md) is a required command line argument.
* The `-a` flag must be passed.
* The configuration settings you require are the same as default, or you set the appropriate environment variables, or you have a `.env` file.

`cryptools` will spit out an error message and then exit/panic if your CSV input file is malformed.
The error message will generally tell you why.
Consider using the python script (root directory of the repo) to sanitize your input file,
in case the file contains negative numbers in parentheses, numbers with commas, or extra rows/columns.

See `/examples/` directory for further guidance,
or jump directly to the [examples.md](https://github.com/scoobybejesus/cryptools/blob/master/examples/examples.md) file.

###### Note: The import of your [CSV input file](https://github.com/scoobybejesus/cryptools/blob/master/InputFile_CSV.md) may fail or behave undesirably with the default configuration settings.
See [.env.example](https://github.com/scoobybejesus/cryptools/blob/master/examples/.env.example) for those defaults.
If you wish to skip the wizard but require changes to default settings, copy `.env.example` to `.env` and make your changes.
The `.env` file must be placed in the directory from which `cryptools` is run or a parent directory.
Alternatively, the respective environment variables may be set manually,
or it may be easier to choose the proper command line flag (such as `-d` for `date_separator_is_slash` or `-i` for `iso_date`.).

#### Pro Tip

Hop into `/usr/local/bin`, and run `ln -s /path/to/cryptools/target/debug/cryptools cryptools`
and `ln -s /path/to/cryptools/clean_input_csv.py clean_input_csv` to be able to run the sanitizer
script and `cryptools` from the directory where you keep your CSV Input File.

## Development state

As of fall 2020, the code does not require additional features in order for it to serve the project's founder.
At the same time, there are still bells and whistles, creature comforts, etc. that are desired and may be added.
Additionally, the code could use factoring or general re-working in several areas.

The software has been tested on Mac, Linux, and FreeBSD.
Additional testers/users are encouraged and welcome.

## Contributing

* Contributors welcome. New authors should add themselves to the `AUTHORS` file.

* Roadmap and todos: we're slowly working through items in [Issues](https://github.com/scoobybejesus/cryptools/issues);
feel free to tackle or add issues.

## A few words from the founder

I have an accounting background, I live in the US, and I am interested in cryptocurrencies.
When it came time to file my tax return, I had to come to grips with recording my cryptocurrency activity.
I initially used a spreadsheet that manually processed the activity into lots, and it quickly became clear that I needed a software solution.
Given my background, I had certain expectations about what this type of software would do, and I tried several online options.
I eventually created this project as a reaction to the inadequate tooling I found online.
Sure, other products have more bells and whistles, but at least I know this produces correct results
(i.e., this software specifically identifies and track all acquired assets, whereas online solutions seems to pool them together).

I am not a formally trained programmer, however I have come to enjoy it very much and I learn more whenever I can.
I originally tried to learn C++ by myself, and that was frustrating.
My first real progress was with Python, but I still didn't manage to fully develop a working program.
Luckily, I managed to stumble across a mentor who helped me write 80% of an MVP in strongly-typed Swift.
We coded our way into a corner, but I had learned enough to take the code apart and put it back together correctly and complete it.
I really enjoyed Swift, but I wanted something even more performant (and cross-platform), and Rust seemed to fit the bill.
I rewrote the code in Rust (also with a bit of help), and it has turned out to be a great choice.

## Legal

See LEGAL.txt