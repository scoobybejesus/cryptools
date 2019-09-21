# cryptools

### Accounting library for cryptocurrency transaction activity.

###### (Currently runs as a binary)

The software measures one's cryptocurrency activity (i.e., denominates one's income/expenses/gains/losses) in their home currency.
The default home currency is USD, but anything can be substituted.
This tool is probably most useful when filing one's taxes.

---

Given a [CSV input file](https://github.com/scoobybejesus/cryptools/blob/master/InputFile_CSV.md)
containing the user's entire cryptocurrency transaction history, the software will:

* assign cost basis as of the date of purchase/exchange/receipt
* track the original acquisition date and cost basis until the time of disposal
* compute gain or loss from the sale/exchange/disposal (including whether short-term or long-term)
* record income for incoming transactions and expenses for outgoing transactions

Reports reflecting income/expenses/gains/losses or amount and cost basis of existing holdings may be exported as CSV/TXT files.

---

#### Features

* Two methods each of LIFO or FIFO (with intentions to add more)

* Ability to perform like-kind exchange treatment

* Compatible with any home currency

#### Constraints

* *All* cryptocurrency-related activity for the user generally must be included in the
[CSV input file](https://github.com/scoobybejesus/cryptools/blob/master/InputFile_CSV.md).

* There can only be either one or two accounts used in a given transaction
(i.e., if a Counterparty token or Ethereum token transaction must be recorded,
the XCP or ETH transaction fee must be reflected in a separate transaction row).

* Manual adjustments may need to be made to the output files in cases, for example,
when the user used appreciated cryptocurrency to make a tax-deductible charitable contribution.

## Installation

1. `cargo build` (include `--release` for a non-debug build)

This will build `./target/debug/cryptools` (or `./target/rls/cryptools` for a non-debug build).

## Usage

Run `./target/debug/cryptools` with no arguments (or `--help`, or `-h`) to see usage.
Alternatively, run `cargo run`, in which case command-line arguments for `cryptools` may be entered following `--`, e.g., `cargo run -- -h`.

Running with no arguments will lead the user through a wizard, or all required arguments can be passed as command-line flags/options/args.
See `/examples/` directory for further guidance,
or jump directly to the [examples.md](https://github.com/scoobybejesus/cryptools/blob/master/examples/examples.md) file.

## Development state

As of summer 2019, the code does not *require* additional features in order for it to serve the project's founder.
At the same time, there are plenty of bells and whistles, creature comforts, etc. that are desired and may be added.
Additionally, the code could use factoring or general re-working in several areas.
In fact, it may be nice to use the business logic as a library instead of running a full binary.

The software has been tested on Mac and Linux.
Additional testers/users are encouraged and welcome.

## Contributing

* Contributors welcome. New authors should add themselves to the `AUTHORS` file.

* Roadmap and todos: we're working through items in [Issues](https://github.com/scoobybejesus/cryptools/issues); feel free to tackle or add issues.

## A few words from the founder

I have an accounting background, and I live in the US.
I took an interest in cryptocurrency in 2016.
When it came time to file my tax return, I had to come to grips with recording my cryptocurrency activity.
I initially used a spreadsheet to manually process the activity into lots, and it quickly became clear that I needed a software solution.
Given my background, I had certain expectations about what this type of software would do, and I tried several online options.
I eventually created this project as a reaction to the inadequate tooling I found online.
Sure, other products have more bells and whistles, but at least I know this produces correct results
(i.e., this software specifically identifies and track all acquired assets, whereas online solutions seems to pool them together).

I am not a formally trained programmer, however I enjoy it very much and learn more whenever I can.
I originally tried to learn C++ by myself, and that was frustrating.
My first real progress was with Python, but I still didn't manage to fully develop a working program.
Luckily, I managed to stumble across a mentor who helped me write 80% of an MVP in strongly-typed Swift.
We coded our way into a corner, but I had learned enough to take the code apart and put it back together correctly and complete it.
I really enjoyed Swift, but I wanted something even more performant (and cross-platform), and Rust seemed to fit the bill.
I rewrote the code in Rust (also with a bit of help from my mentor), and it has turned out to be a great choice.

## Legal

See LEGAL.txt
