# cryptools

### Accounting library for cryptocurrency transaction activity.

###### (Currently runs as a binary, not a library)

The software measures one's cryptocurrency activity (i.e., denominates one's income/expenses/gains/losses) in their home currency.
The default home currency is USD, but anything can be substituted.
This type of tool may be useful, for example, when preparing to file taxes.

The software has been tested on Mac and Linux.
Testers/users are encouraged and welcome.

---

Given an input CSV file reflecting the user's entire cryptocurrency transaction history, the software will:

* assign cost basis as of the date of purchase/exchange/receipt
* track the original acquisition date and cost basis until the time of disposal
* compute gain or loss from the sale/exchange/disposal (including whether short-term or long-term)
* record income for incoming transactions and expenses for outgoing transactions

Read up on the [CSV input file](https://github.com/scoobybejesus/cryptools/blob/master/InputFile_CSV.md).

Reports may be exported (as CSV files) that reflect income/expenses/gains/losses or amount and cost basis of existing holdings.

---

#### Features

* Two methods each of LIFO or FIFO (with intentions to add more)

* Ability to perform like-kind exchange treatment

* Compatible with any home currency

#### Constraints

* *All* cryptocurrency-related activity for the user generally must be included in the input CSV file.

* There can only be either one or two accounts used in a given transaction
(i.e., if a Counterparty token or Ethereum token transaction must be recorded,
the XCP or ETH transaction fee must be reflected in a separate transaction row).

* Manual adjustments may need to be made to the output files in cases, for example,
when the user used aprpeciated cryptocurrency to make a tax-deductible charitable contribution.

## Installation

1. `cargo build` (or include `--release` for a non-debug build)

This will build `./target/debug/cryptools` (or `./target/rls/cryptools` for a non-debug build).

## Usage

Run `./target/debug/cryptools` with no arguments (or `--help`, or `-h`) to see usage.
Alternatively, run `cargo run`, in which case command-line arguments for `cryptools` may be entered following `--`, e.g., `cargo run -- -h`.

Running with no arguments will lead the user through a wizard, or all required arguments can be passed as command-line flags/options/args.
See `/examples/` directory for further guidance,
or jump directly to the [examples.md](https://github.com/scoobybejesus/cryptools/blob/master/examples/examples.md) file.

## Development state

As of summer 2019, the code is "feature complete" in the sense that it does not require additional features in order for it to serve the needs of the original author.
At the same time, there are plenty of bells and whistles, creature comforts, etc. that may be added at any time.
Additionally, the code could use factoring or general re-working in several areas.
In fact, it may be nice to use the business logic as a library instead of running a full binary.

## Contributing

* Contributors welcome. New authors should add themselves to the `AUTHORS` file.

* Roadmap and todos: we're working through items in [Issues](https://github.com/scoobybejesus/cryptools/issues); feel free to tackle or add issues.

## Legal

See LEGAL.txt
