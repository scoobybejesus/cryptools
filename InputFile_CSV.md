## Overview

The key to understanding how to use this software is to understand the input file.
This document aims to tell the user all they need to know.

Note: the input file is needed in the *current* version of the software.
Future versions may store the transactional activity in a database or similar file (though there are no current plans for this).
At that time, it may be that additional transactions are entered directly into the software.
For now, however, additional transactions must be added to the input file,
and then the input file is imported ***in full*** every time the software is run.

## Preface

`cryptools` is a command-line utility that does not store data.
It is not a full-featured bookkeeping tool that stores your transactions in a persistent ledger.
You cannot open the software later, load the ledger, and enter new transactions.

Rather, you must maintain the input file outside of `cryptools`.
If you have new transactions, you must append them to the input file.
Once you have added new transactions to the input file and wish to view updated reports,
you must run the software again and import the recently-updated input file.
New reports will be generated at that time.

When `cryptools` runs, it imports the input file, processes the transactions, and prints reports (all, or those you choose).
The transaction reports reflect income, expenses, realized gains, and realized losses (and holding period).
At a minimum, the account reports reflect account balances and cost basis of holdings.
Some account reports reflect *all* movements in *every* lot in *every* account, including tracking of cost basis and more.

After the reports have been printed/exported, the software stops running and the memory is abandoned/cleared.
Once the software has run, you rely on the reports to tell you everything you need to know.

### Why should I go to the trouble?

Think about this: cryptocurrency exchanges have **no way** to track the cost basis of their users' holdings,
nor can they verify how the funds they disburse are used.
At most, an exchange will be able to provide the gross value of their transactions (what `cryptools` calls `proceeds`).
If exchanges provide a user's trading activity to the IRS, for example, gross value is all they can provide.
Consequently, unless you prove otherwise, the IRS will assume all trades and all disbursements are taxable income/gains.

With that in mind, aggregating one's entire crypto history is very important!
Cryptocurrency users **cannot** rely on exchanges to provide them (or the IRS) with gain/loss information.
This means it is **up to users** to keep track of their cost basis.

By aggregating all your cryptocurrency activity, this software enables you to track **and prove** your cost basis.
Using these reports, you can walk back in time, tracing the history of the cost basis from every gain/loss calculation.
(If you paid cash for any cryptos, proof becomes much more difficult, but at least you have an otherwise complete ledger.)
The hurdle, of course, is preparing the input file.

### A final note before digging in

The input file *can be* intimidating.
Admittedly, some people will have a tough time preparing it correctly or at all.
The trouble tends to come when aggregating a user's entire crypto history long after the fact.
Even so, preparing and maintaining the input file is **not** very complicated.
There are a small, limited number of rules to follow, and that's it.
The truth is that the input file is simple to maintain once it is brought current and kept current.

### Rules of the input file

The rules for successfully preparing and maintaining the input file can generally be summarized as follows:

1. The first account must be given number `1`, and each additional account must count up sequentially.
2. `Proceeds` is the value of the transaction (measured in the home currency), whether spent, received, or exchanged.
It is **required** in order to properly calculate income/expense/gain/loss.
3. `Proceeds` must have a period as the decimal separator (`1,000.00` not `1.000,00`) and must not contain the ticker or symbol (USD or $).
4. Margin quote account `ticker`s must be followed by an underscore and the base account ticker (i.e., `BTC_xmr`).
5. Only home currency accounts can have negative balances. Non-margin crypto accounts may not go negative at any time.
(Exception: crypto margin accounts may go negative.)

As you can see, most of the rules can generally be ignored.
In fact, the only tricky field is the `proceeds` column, but even that becomes second nature soon enough.

Keep an eye out for a related project that creates input file pro formas from exchange reports, thus automating some of the process.

## Visual representation

In order to be successfully imported, the CSV input file **must** be in a prescribed form that effectively looks like this:


|txDate |proceeds|memo        |1     |2       |3      |4       |5           |
|-------|--------|------------|------|--------|-------|--------|------------|
|       |        |            |Bank  |Exchange|Wallet |Exchange|Simplewallet|
|       |        |            | USD  | BTC    | BTC   | XMR    | XMR        |
|       |        |            | non  | non    | non   | non    | non        |
|2-1-16 |0       |Bought      |  -220|    0.25|       |        |            |
|3-1-16 |250     |Traded      |      |   -0.25|       |     180|            |
|4-1-16 |0       |Transferred |      |        |       |     -90|          90|
|5-1-16 |0       |Transferred |      |        |       |      90|         -90|
|5-2-16 |160     |Traded      |      |     0.3|       |     -90|            |
|6-1-16 |0       |Transferred |      |    -0.3|    0.3|        |            |
|7-1-16 |200     |Traded      |      |     0.7|       |     -90|            |
|8-1-16 |0       |Transferred |      |     0.3|   -0.3|        |            |
|9-1-16 |400     |Traded      |      |    -0.5|       |     200|            |
|10-1-16|900     |Traded      |      |       1|       |    -200|            |
|11-1-16|0       |Transferred |      |    -1.5|    1.5|        |            |
|12-1-16|2000    |Traded*     |      |        |   -1.5|        |         400|

---

### CSV file components - What they are

##### Columns

The first three columns (ignoring the first four rows) are for transaction metadata.

* **txDate**: With each row being a transaction, this is the date of the transaction in that row.

* **proceeds**: This is either (a) the value transferred from one party to another in a transaction,
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

* **memo**: is a description of the transaction.
This should be used to describe the nature of, or reason for, the transaction.
This could be important for remembering, for example, a tax-deductible payment,
or maybe for distinguishing between mining earnings and a customer deposit.
A memo is also useful when evaluating the reports you print/export,
because there may be several transactions on the same day and a good memo helps you identify them.

* *Accounts*: After three columns of transaction metadata, the *Account* columns follow.
The increases and decreases to each account are recorded directly below in that account's column
as part of the transaction activity.

##### Rows

The first four rows (ignoring the first three columns) are for account metadata.

* *Account number* (**1**, **2**, **3**, **4**, **5**, ...): the top row reflects the account number
(which currently must start at 1 and increase sequentially).

* *Name*: This second row is the [friendly] name of the wallet or exchange.

* *Ticker*: This should be self-explanatory (i.e., USD, EUR, BTC, XMR, ETH, LTC, etc.).

* *Margin_bool*: This describes, in 'yes' or 'no' fashion, whether the account is a margin account.

* *Transactions*: After four header rows to describe the accounts, the transaction rows follow.
Each row reflects the net effect on the accounts involved, net of any fees.
There must be a value associated with *either* one *or* two accounts in each transaction row,
depending on the type of transaction, i.e.:
    * a deposit will reflect a positive value posting to the account where money was deposited
    * a payment will reflect a negative value posting to the account from where the money was spent
    * an exchange would reflect a negative value in one account and a positive value in another (differing `ticker`s)
    * a transfer (toSelf) would also reflect a negative value in one account and a positive value in another (same `ticker`)

(An exchange transaction will trigger a gain or loss, whereas a toSelf transfer would not.)

A note on transaction/network fees and exchange fees:

In the first transaction above, 0.25 BTC was received,
but the purchase would probably really have been buying, say, 0.25002.
That extra 0.00002 is a transaction fee kept by the exchange, so we ignore it.
The 0.25 represents the increase in the BTC balance, so 0.25 is used.

The same logic applies to the next transaction.
0.25 BTC was used to buy XMR, and the XMR balance increased by 180 as a result.
But the user would have paid a transaction fee, so the exchange rate suggested the amount paid for was really, say, 180.002.
In fact, the user would literally have placed an order for 180.002 at a price where they'd pay 0.25.
Or, if the fee is paid in BTC, the order might have been .2498 BTC for 180 XMR, but the total cost is .25 BTC because of a .0002 BTC fee.
The details regarding the fee are irrelevant, however, when it comes to entering the trade in the input file
because this software only cares about what you pay and what you receive in exchange as a result of the transaction.

Looking at the third and fourth transactions, the same amount was withdrawn from one account and deposited into another account.
This may be an unrealistic scenario because network transaction fees and exchange transfer fees are ignored.
There are exchanges who will cover the transfer fee, but it's not common.
It certainly is unrealistic for the user to send from their personal wallet (in the fourth transaction) with no network fee.
These transactions are oversimplified on purpose.
If you transfer 0.1 BTC from one account to another, and the other account receives 0.0999, then that's what you record.

###### Margin accounts

* Margin accounts always come in pairs, the base account and the quote account.

* Margin gain or loss is accounted for when there is activity in the related "spot" account.
For example, a loss will not be recorded until "spot" holdings are used to pay off loans.
Until "spot" funds are spent to pay off the margin loans, it's simply an [unrecorded] unrealized loss.

### CSV file components - Data types, restrictions, and important points

##### Columns

* **txDate**: As a default, this parser expects a format of `MM-dd-YY` or `MM-dd-YYYY`.
The ISO 8601 date format (`YYYY-MM-dd` or `YY-MM-dd` both work) may be indicated by setting the environment variable `ISO_DATE` to `1` or `true`.
The hyphen date separator character (`-`) is the default.  The slash date separator character (`/`) may be indicated
by setting the `DATE_SEPARATOR_IS_SLASH` environment variable (or in .env file) to `1` or `true`,
or by passing the `date_separator_is_slash` command line flag.

* **proceeds**: This is can be any **positive** number that will parse into a floating point 32-bit number,
as long as the **decimal separator** is a **period**.
The software is designed to automatically *remove any commas* prior to parsing.
The value in this field is denominated in the user's **home currency**,
but be sure not to include the ticker or symbol of the currency
(i.e., for `$14,567.27 USD`, enter `14567.27` or `14,567.27`).

* **memo**: This can be a string of characters of any length, though fewer than 20-30 characters is advised.

* *quantity*: This is similar to **proceeds**, in that the **decimal separator** must be a **period**,
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
For each transaction, input the **date**, **proceeds**, **memo**, and **quantity** by which the account balances change.
As mentioned elsewhere, a minimum of one and a maximum of two **Accounts** can be associated with a single transaction.