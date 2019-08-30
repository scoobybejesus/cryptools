# cryptools-rs

### Accounting library for cryptocurrency transaction activity.

###### (Currently runs as a binary, not a library)

The software measure one's cryptocurrency activity (i.e., denominates one's income/expenses/gains/losses) in their home currency.
The default home currency is USD, but anything can be substituted.
This type of tool may be useful, for example, when preparing to file taxes.

Given an input CSV file reflecting the user's entire cryptocurrency transaction history, the software will:

* assign cost basis as of the date of purchase/exchange/receipt
* track the original acquisition date and cost basis until the time of disposal
* compute gain or loss from the sale/exchange/disposal (including whether short-term or long-term)
* record income for incoming transactions and expenses for outgoing transactions

Reports (CSV file) may be exported that reflect income/expenses/gains/losses or cost basis of existing holdings.

---

The activity that gets imported **must** be in a prescribed form (a CSV file) that effectively looks like this:


|txDate |proceeds|memo    |1     |2       |3      |4       |5           |
|-------|--------|--------|------|--------|-------|--------|------------|
|       |        |        |Bank  |Exchange|Wallet |Exchange|Simplewallet|
|       |        |        |USD   |BTC     |BTC    |XMR     |XMR         |
|       |        |        |non   |non     |non    |non     |non         |
|2/1/16 |0       |FIRST   |-220  |0.25    |       |        |            |
|3/1/16 |250     |SECOND  |      |-0.25   |       |180     |            |
|4/1/16 |0       |THIRD   |      |        |       |-90     |90          |
|5/1/16 |0       |FOURTH  |      |        |       |90      |-90         |
|5/2/16 |160     |FIFTH   |      |0.3     |       |-90     |            |
|6/1/16 |0       |SIXTH   |      |-0.3    |0.3    |        |            |
|7/1/16 |200     |SEVENTH |      |0.7     |       |-90     |            |
|8/1/16 |0       |EIGHTH  |      |0.3     |-0.3   |        |            |
|9/1/16 |400     |NINTH   |      |-0.5    |       |200     |            |
|10/1/16|900     |TENTH   |      |1       |       |-200    |            |
|11/1/16|0       |ELEVENTH|      |-1.5    |1.5    |        |            |
|12/1/16|2000    |TWELFTH*|      |        |-1.5   |        |400         |

\* This last transaction is an example of how a user might reflect an exchange via Shapeshift or similar services, where one currency is spent from one wallet and a different currency is received in another wallet.

---

#### CSV file components

* **txDate** is currently set to parse dates of the format MM/dd/YY.

* **proceeds** This is the value transferred in the transaction.
For example, if one spends 0.01 BTC at a time when BTC/USD is $10,000/BTC, then the user received value of $100, therefore the proceeds of that transaction would be $100.
When transfering to oneself (i.e.,  not changing currencies), this value is irrelevant and ignored.
This value is also ignored when the user's home currency is spent.

* **memo** is useful for evaluating the final output but isn't important.
Currently, commas in the memo are **not** supported.

After three columns of transaction metadata, the *Account* columns follow.

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

* Margin gain or loss is accounted for when there is activity in the 'spot' account.
For example, you won't reflect a loss until you actually spend your holdings to pay off your loans.

---

#### Features

* Two methods each of LIFO or FIFO (with intentions to add more)

* Ability to perform like-kind exchange treatment

#### Constraints

* *All* cryptocurrency-related activity for the user generally must be included in the input CSV file.

* There can only be either one or two accounts used in a given transaction (i.e., if a Counterparty token or Ethereum token transaction must be recorded, the XCP or ETH transaction fee must be reflected in a separate transaction/row).

* Manual adjustments may need to be made to the output files in cases, for example, when the user used aprpeciated cryptocurrency to make a tax-deductible charitable contribution.

## Installation

1. `cargo build` (or include `--release` for a non-debug build)

This will build `./target/debug/cryptools-rs`.

## Usage

Run `./target/debug/cryptools-rs` with no arguments (or `--help`, or `-h`) to see usage.
Alternatively, run `cargo run`, in which case command-line arguments for `cryptools-rs` may be entered following `--`, e.g., `cargo run -- -h`.

Running with no arguments will lead the user through a wizard, or all required arguments can be passed as command-line flags/options/args.

## Development state

As of summer 2019, the code is "feature complete" in the sense that it does not require additional features in order for it to serve the needs of the original author.
At the same time, there are plenty of bells and whistles, creature comforts, etc. that may be added at any time.
Additionally, the code could use factoring or general re-working in several areas.
In fact, it may be nice to use the business logic as a library instead of running a full binary.

## Contributing

* Contributors welcome. New authors should add themselves to the `AUTHORS` file.

* Roadmap and todos: we're working through items in [Issues](https://github.com/scoobybejesus/cryptools-rs/issues); feel free to tackle or add issues.

## Legal

See LEGAL.txt
