# cryptools-rs

### Accounting library for cryptocurrency transaction activity.

It provides a way to measure cryptocurrency activity in one's home currency (the default value is USD, but anything can be used).
Reports may be exported as CSV files that reflect income/expense/gains/losses.

The activity that gets imported **must** be in a prescribed form that effectively looks like this:


|txDate |proceeds|memo    |1     |2       |3       |4       |5           |
|-------|--------|--------|------|--------|--------|--------|------------|
|       |        |        |Bank  |Exchange|Exchange|Exchange|Simplewallet|
|       |        |        |USD   |BTC     |BTC     |XMR     |XMR         |
|       |        |        |non   |non     |non     |non     |non         |
|2/1/16 |0       |FIRST   |-220  |0.25    |        |        |            |
|3/1/16 |250     |SECOND  |      |-0.25   |        |180     |            |
|4/1/16 |0       |THIRD   |      |        |        |-90     |90          |
|5/1/16 |0       |FOURTH  |      |        |        |90      |-90         |
|5/2/16 |160     |FIFTH   |      |0.3     |        |-90     |            |
|6/1/16 |0       |SIXTH   |      |-0.3    |0.3     |        |            |
|7/1/16 |200     |SEVENTH |      |        |0.7     |-90     |            |
|8/1/16 |0       |EIGHTH  |      |0.5     |-0.5    |        |            |
|9/1/16 |400     |NINTH   |      |        |-0.5    |200     |            |
|10/1/16|900     |TENTH   |      |1       |        |-200    |            |
|11/1/16|0       |ELEVENTH|      |-1.5    |1.5     |        |            |
|12/1/16|2000    |TWELFTH |      |        |-1.5    |400     |            |


#### CSV file components

* **txDate** is currently set to parse dates of the format MM/dd/YY.

* **proceeds** may seem tricky.
The way to understand it, since it can apply to any transaction (aside from transfers from one owned-account to another owned-account), is that this is the value transferred in the transaction.
For example, if one spends 0.01 BTC for an item at a time when BTC/USD is $10,000/BTC, then the user received value of $100, therefore the proceeds of that transaction would be $100.
This field is ignored when the user's home currency is used to purchase cryptocurrency.

* **memo** is useful for evaluating the final output but isn't important.
Currently, commas in the memo are **not** supported.

After three column of transaction metadata, the *Account* columns follow.

* *Accounts* (**1**, **2**, **3**, **4**, **5**, ...): the top row reflects the account number (which currently must start at 1 and increase sequentially).
The three other values are the *name*, *ticker*, and *margin_bool*.
*name* and *ticker* should be self-explanatory.
*margin_bool* is set usually set as 'no', 'non' (i.e., non-margin), or 'false.'
To indicate a margin account, set it as 'yes', 'margin' or 'true'.

###### Margin accounts

* Margin accounts must come in pairs, the base account and the quote account.
The base account is the coin being longed or shorted, and its ticker is reflected like normal.
The quote account is the market.
The quote account's ticker requires a different formatting.
For example, when using BTC to long XMR, the BTC account must be reflected with the ticker BTC_xmr.

#### Constraints

* *All* cryptocurrency-related activity for the user generally must be included in the input CSV file.

* There can only be either one or two accounts used in a given transaction (i.e., if a Counterparty token or Ethereum token transaction must be recorded, the XCP or ETH transaction fee must be reflected in a separate transaction/row).

* Currently, manual adjustments may need to be made to the output files in cases, for example, when the user used cryptocurrency to make atax-deductible charitable contribution.

## Installation

1. `cargo build` (or include `--release` for a non-debug build)

This will build `./cryptools-rs`.

## Usage

Run `./target/debug/cryptools-rs` with no arguments (or `--help`, or `-h`) to see usage.
Alternatively, run `cargo run`, in which case command-line arguments for `cryptools-rs` may be entered following `--`, e.g., `cargo run -- -h`.

## Contributing

* Contributors welcome. New authors should add themselves to the `AUTHORS` file.

* Roadmap and todos: we're working through items in [Issues](https://github.com/scoobybejesus/cryptools-rs/issues); feel free to tackle or add issues.

## Legal

See LEGAL.txt
