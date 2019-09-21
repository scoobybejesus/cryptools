// Copyright (c) 2017-2019, scoobybejesus

// Redistributions must include the license: https://github.com/scoobybejesus/cryptools/blob/master/LEGAL.txt

---

**`input_file.csv`**

The key to understanding the value in this software is to understand the input file.
All income, expenses, gains, and losses (and related like-kind treatment)
are deterministically calculated based on the input file (and the command-line parameters).
This document aims to tell a user all they need to know about the CSV input file.


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
