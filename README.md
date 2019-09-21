# cryptools

### Accounting library for cryptocurrency transaction activity.

###### (Currently runs as a binary, not a library)

The software measures one's cryptocurrency activity (i.e., denominates one's income/expenses/gains/losses) in their home currency.
The default home currency is USD, but anything can be substituted.
This type of tool may be useful, for example, when preparing to file taxes.

Given an input CSV file reflecting the user's entire cryptocurrency transaction history, the software will:

* assign cost basis as of the date of purchase/exchange/receipt
* track the original acquisition date and cost basis until the time of disposal
* compute gain or loss from the sale/exchange/disposal (including whether short-term or long-term)
* record income for incoming transactions and expenses for outgoing transactions

Reports may be exported (as CSV files) that reflect income/expenses/gains/losses or amount and cost basis of existing holdings.

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

##### Columns

The first three columns (ignoring the first four rows) are for transaction metadata.

* **txDate** is currently set to parse dates of the format MM/dd/YY.

* **proceeds** This is either (a) the value transferred from one party to another in a transaction,
or (b) the value exchanged in a trade (an exchange transaction).
In both cases, the value is measured in one's home currency!
For example, if the user spends 0.01 BTC at a time when BTC/USD is $10,000/BTC,
and in exchange for that 0.01 BTC the user receives something valued at $100,
the proceeds of that transaction (despite it being an outflow) would be $100.
Similarly, for the user receiving the 0.01 BTC (as an inflow), they would reflect proceeds of $100 as well.
And if the recipient immediately spent the 0.01 BTC in exchange for XMR,
the proceeds of that transaction would also be $100.
Note: when transfering to oneself (i.e.,  not changing currencies), the proceeds value is irrelevant and ignored.
This value is also ignored when the user's home currency is spent.

* **memo** is useful for evaluating the final output but isn't important.
Currently, commas in the memo are **not** supported.

* *Accounts* - After three columns of transaction metadata, the *Account* columns follow.
The increases and decreases to each account are recorded directly below in that account's column.

##### Rows

The first four rows (ignoring the first three columns) are for account metadata.

* *Account number* (**1**, **2**, **3**, **4**, **5**, ...): the top row reflects the account number (which currently must start at 1 and increase sequentially).

* *Name* is the name of the wallet or exchange.

* *Ticker* should be self-explanatory (i.e., USD, EUR, BTC, XMR, ETH, LTC, etc.).

* *Margin_bool* is usually set as "no", "non" (i.e., non-margin), or "false".
To indicate a margin account, set it as "yes", "margin" or "true".

* *Transactions*: After four header rows to describe the accounts, the transaction rows follow.
Each row reflects the net effect on the accounts involved, net of any fees.

For example, in the first transaction above, 0.25 BTC was received,
but the purchase would probably really have been for, say, 0.25002.
That 0.00002 is a transaction fee kept by the exchange, so we ignore it.
The 0.25 represents the increase in the BTC balance, so 0.25 is used.

The same logic applies to the next transaction.
0.25 BTC was used to buy XMR, and the XMR balance increased by 180 as a result.
But the user would have paid a transaction fee, so the amount paid for was really, say, 180.002.
In fact, the user would literally have placed an order for 180.002 at a price where they'd pay 0.25.
It doesn't matter, though, that the order was placed for 180.002
because this software only cares about what you receive in your wallet as a result of the transaction.

Looking at the third and fourth transactions, the same amount was withdrawn from one account and deposited into another account.
This is an unrealistic scenario because network transaction fees and exchange transfer fees are ignored.
There are exchanges who will cover the transfer fee, but it's rare.
And it certainly is unrealistic for the user to send from their personal wallet (in the fourth transaction) with no network fee.
These transactions are oversimplified on purpose.

###### Margin accounts

* Margin accounts must come in pairs, the base account and the quote account.
The base account is the coin being longed or shorted, and its ticker is reflected like normal.
The quote account is the market.
The quote account's ticker requires a different formatting.
For example, when using BTC to long XMR, the BTC account must be reflected with the ticker BTC_xmr.

* Margin gain or loss is accounted for when there is activity in the related "spot" account.
For example, you won't reflect a loss until you actually spend your holdings to pay off your loans.
Until you sell, it's simply an unrealized loss.

---

#### Features

* Two methods each of LIFO or FIFO (with intentions to add more)

* Ability to perform like-kind exchange treatment

* Compatible with any home currency

#### Constraints

* *All* cryptocurrency-related activity for the user generally must be included in the input CSV file.

* There can only be either one or two accounts used in a given transaction (i.e., if a Counterparty token or Ethereum token transaction must be recorded, the XCP or ETH transaction fee must be reflected in a separate transaction/row).

* Manual adjustments may need to be made to the output files in cases, for example, when the user used aprpeciated cryptocurrency to make a tax-deductible charitable contribution.

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
