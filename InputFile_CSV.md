## Overview

The key to understanding how to use this software is to understand the input file.
This document aims to tell the user all they need to know.

Note: the input file is needed in the *current* version of the software.
Future versions may store the transactional activity in a database or similar file.
At that time, it may be that additional transactions are entered directly into the software.
For now, however, additional transactions must be added to the input file,
and then the input file is imported ***in full*** every time the software is run.

## Preface

`cryptools` is a command-line utility that does not store data.
It is not a bookkeeping tool.
It does not store your transactions in a persistent ledger.
You cannot open the software later, load the ledger, and enter new transactions.

Rather, you must maintain the input file outside of `cryptools`.
If you have new transactions, you must append them to the input file.
Once you have added new transactions to the input file and wish to view updated reports,
you must run the software again and import the recently-updated input file.
New reports will be generated at that time.

When `cryptools` runs, it imports the input file, it processes the transactions, and it prints reports.
The transaction reports reflect income, expenses, realized gains, and realized losses.
At a minimum, the account reports reflect account balances and cost basis of holdings.
Some account reports reflect *all* movements in *every* lot in *every* account, including tracking of cost basis and more.

After the reports have been printed/exported, the software stops running and the memory is abandoned/cleared.
Once the software has run, you rely on the reports to tell you everything you need to know.

### Why should I go to the trouble?

Think about this: exchanges have **no way** to track the cost basis of their users' holdings,
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

#### Rules of the input file

The rules for successfully preparing and maintaining the input file can generally be summarized as follows:

1. The first account must be given number `1`, and each additional account must count up sequentially.
2. Margin quote account `ticker`s must be followed by an underscore and the base account ticker (i.e., `BTC_xmr`).
3. `Proceeds` is the value of the transaction, whether spent, received, or exchanged.
It is **required** in order to properly calculate income/expense/gain/loss.
4. `Proceeds` must have a period as a decimal separator (`1,000.00` not `1.000,00`) and must not contain the ticker or symbol (USD or $).
5. Only home currency accounts can have negative balances. Crypto accounts may not go negative at any time.
(Exception: crypto margin accounts may go negative, of course.)

As you can see, most of the rules can generally be ignored.
In fact, the only tricky field is the `proceeds` column, but even that becomes second nature soon enough.

Keep an eye out for a related project that creates input file pro formas from exchange reports, thus automating some of the process.

## Visual representation

In order to be successfully imported, the CSV input file **must** be in a prescribed form that effectively looks like this:


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

* Margin accounts always come in pairs, the base account and the quote account.

* Margin gain or loss is accounted for when there is activity in the related "spot" account.
For example, a loss will not be recorded until "spot" holdings are used to pay off loans.
Until "spot" funds are spent to pay off the margin loans, it's simply an [unrecorded] unrealized loss.

### CSV file components - Data types, restrictions, and important points

##### Columns

* **txDate**: As a default, this parser expects a format of `MM-dd-YY` or `MM-dd-YYYY`.
The ISO 8601 date format (`YYYY-MM-dd` or `YY-MM-dd` both work) may be indicated by passing the `-i` flag.
The hyphen, slash, or period delimiters (`-`, `/`, or `.`) may be indicated
by passing the `-d` option followed by `h`, `s`, or `p`, respectively (hyphen, `-`, is default).

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