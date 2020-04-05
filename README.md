# cryptools

### Accounting library for cryptocurrency transaction activity.

###### (The package produces a binary and accompanying library)

This is a command-line tool that calculates income, expenses, realized gains, and realized losses
(and holding period) from cryptocurrency activity and denominates the results in the user's home currency.
The default home currency is USD, but any currency can be substituted.
This tool is probably most useful for filling out a tax return or making tax planning decisions.

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

---

### Features

* Two methods each of LIFO or FIFO (compatible w/ the concept of "specific identification")

* Ability to perform like-kind exchange treatment

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

## Installation

1. `git clone https://github.com/scoobybejesus/cryptools.git`
2. `cd cryptools`
3. `cargo build` (include `--release` for a non-debug build)

This will build `./target/debug/cryptools` (or `./target/release/cryptools` for a non-debug build).

## Usage

Run `./target/debug/cryptools` with no arguments (or with `--help`, or `-h`) to see usage.
Alternatively, run `cargo run`, in which case command-line arguments for `cryptools` may be entered following `--`, e.g., `cargo run -- -h`.

Running with no arguments will lead the user through a wizard; or all required arguments can be passed as command-line flags/options/args.
See `/examples/` directory for further guidance,
or jump directly to the [examples.md](https://github.com/scoobybejesus/cryptools/blob/master/examples/examples.md) file.

## Development state

As of summer 2019, the code does not require additional features in order for it to serve the project's founder.
At the same time, there are plenty of bells and whistles, creature comforts, etc. that are desired and may be added.
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
I rewrote the code in Rust (also with a bit of help from my mentor), and it has turned out to be a great choice.

## Legal

See LEGAL.txt
