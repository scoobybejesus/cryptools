// Copyright (c) 2017-2019, scoobybejesus

// Redistributions must include the license: https://github.com/scoobybejesus/cryptools/blob/master/LEGAL.txt

---

**`input_file.csv`**

The key to understanding how to use this software is to understand the input file.
The input file encodes all accounts, including all transactions occurring in and between those accounts.
All income, expenses, gains, and losses (and related like-kind treatment results)
are deterministically calculated based on the input file (and the command-line parameters).
This document aims to tell a user all they need to know about the CSV input file.

Note: the input file is needed in the *current* version of the software.
Future versions may store the transactional activity in a database or similar file.
At that time, additional transactions perhaps may be entered directly into the software.
For now, however, additional transactions are added to the input file,
and then the input file is imported ***in full*** every time the software is run.

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

---

#### CSV file components - What they are

##### Columns

The first three columns (ignoring the first four rows) are for transaction metadata.

* **txDate** With each row being a transaction, this is the date of the transaction in that row.

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

* **memo** is a description of the transaction.
This should be used to describe the nature of, or reason for, the transaction.
This could be important for remembering, for example, a tax-deductible payment,
or maybe for distinguishing between mining earnings and a customer deposit.
A memo is also useful when evaluating the reports you print/export,
because there may be several transactions on the same day and a good memo helps you identify them.

* *Accounts* - After three columns of transaction metadata, the *Account* columns follow.
The increases and decreases to each account are recorded directly below in that account's column
as part of the transaction activity.

##### Rows

The first four rows (ignoring the first three columns) are for account metadata.

* *Account number* (**1**, **2**, **3**, **4**, **5**, ...): the top row reflects the account number (which currently must start at 1 and increase sequentially).

* *Name*: This second row is the [friendly] name of the wallet or exchange.

* *Ticker*: This should be self-explanatory (i.e., USD, EUR, BTC, XMR, ETH, LTC, etc.).

* *Margin_bool*: This describes, in 'yes' or 'no' fashion, whether the account is a margin account.

* *Transactions*: After four header rows to describe the accounts, the transaction rows follow.
Each row reflects the net effect on the accounts involved, net of any fees.
There must be a value associated with *either* one *or* two accounts in each transaction row,
depending on the type of transaction, i.e.:
    * a deposit will reflect a postive value entering the account where money was deposit
    * a payment will reflect a negative value exiting the account from where the money was spent
    * an exchange would reflect a negative value in one account and a positive value in another
    * a transfer (toSelf) would also reflect a negative value in one account and a positive value in another

(An exchange transaction will trigger a gain or loss, whereas a toSelf transfer would not.)

A note on transaction/network fees and exchange fees:

In the first transaction above, 0.25 BTC was received,
but the purchase would probably really have been buying, say, 0.25002.
That extra 0.00002 is a transaction fee kept by the exchange, so we ignore it.
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

#### CSV file components - Data types, restrictions, and important points

##### Columns

* **txDate**: This currently can parse dates of the formats `MM/dd/YY` and `MM/dd/YYYY`.
The forward slash delineator must be used; not a hyphen.
The ISO 8601 date format (`YYYY-MM-dd`) will be implemented eventually, including the hyphen delineator.

* **proceeds**: This is can be any **positive** number that will parse into a floating point 32-bit number,
as long as the **decimal separator** is a **period**.
The software is designed to automatically *remove any commas* prior to parsing.
The value in this field is denominated in the user's **home currency**,
but be sure not to include the ticker or symbol of the currency
(i.e., for `$14,567.27 USD`, enter `14567.27` or `14,567.27`).

* **memo**: This can be a string of characters of any length, though fewer than 20-30 characters is advised.
Currently, **commas** in the memo field are **not** supported.

* *value*: This is similar to **proceeds**, in that the **decimal separator** must be a **period**,
and you *cannot* include the ticker or symbol of the currency in that field.
It is different from **proceeds** in that this will be parsed into a 128-bit precision decimal floating point number,
and a negative value can be indicated via a preceding `-`.
Negative values currently cannot be parsed if they are instead wrapped in parentheses (i.e., `(123.00)`).

##### Rows

* *Account number*: This will be parsed as a positive integer, the first account currently must
be `1`, and each additional Account column must increase this value by `1`.

* *Name*: This can be any string, though it should be very short, ideally.

* *Ticker*: This can also be any string, but be mindful of the special formatting required for the margin quote account.
Consider a scenario where XMR is being margin-traded in BTC terms.
The price would be quoted in terms of the currency pair XMR/BTC, where XMR is the base account and BTC is the quote account.
The software will behave fine with the XMR ticker as `XMR`, but the BTC ticker must be reflected as `BTC_xmr`.
Note the underscore (`_`) that is used to signify that BTC was used to long or short XMR.

* *Margin_bool*: This is usually set as "no", "non" (i.e., non-margin), or "false".
To indicate a margin account, set it as "yes", "margin" or "true".
Anything aside from those six choices will fail to parse.

* *Transactions*: After the four header rows describing the accounts, the transaction rows follow.
Each row is a separate transaction.
For each transaction, input the **date**, **proceeds**, **memo**, and **values** by which the account balances change.
As mentioned elsewhere, a minimum of one and a maximum of two Accounts can be associated with a single transaction.