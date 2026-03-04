# Build And Run Instructions

```shell
cargo build

cargo run -- transactions.csv
```

Use the `-d` / `--debug` flag to see debug logs:

```shell
cargo run -- transactions.csv --debug
```

# Payment System Details

This is a single-threaded payment system solution that processes lists of transactions in CSV format. It uses the `csv`
crate to read each transaction into memory one at a time, `serde` to transform the bytes data into data structures that
are used throughout the payment system, and `rust_decimal` to easily ensure 4 decimal precision.

I used the very popular `clap` crate to expose the CLI interface. One could argue that clap is overkill given that there
is only 1 required argument and that I could have just used `std::env::args()` instead. But I opted to use clap anyways
for its convenience and potential for extension in the future. For instance, it enabled me to easily add a new optional
argument for showing debug logs.

Speaking of which, I added debug/error statements throughout the program via `log` and `env_logger`. Although this makes
the code slightly more bloated, I thought it would be worth keeping in to help catch bugs and verify the program is
running as I expect it to. This feature is disabled by default as required by the specifications.

In order to process the transactions efficiently, I used 2 in-memory databases (`BTreeMap`s) to dynamically track the
updates to Client Accounts after applying each transaction. One of the databases is used to remember previous
transactions so that the payment system can prevent invalid scenarios like duplicate transactions. In a real-world
payment system, these databases are almost certainly remote ones that require extra machinery to ensure strongly
consistent reads/writes.

This payment system uses the Transaction's type as a sort of state machine to know what are valid future transactions on
the same TransactionId. There is essentially 2 simple state machines. The first is just Withdrawal which is completely
standalone. The second is the Deposit state machine which can flow into dispute-related states
(Dispute/Resolve/Chargeback).

## Out Of Scope

There were a number of things I wanted to build into this system, namely async, traits + generics, and type-enforced
state machines. For simplicity and time, I opted to not pursue these endeavors.

Converting this implementation to async is possible but will require non-trivial work to ensure strongly consistent
access to the databases so that tasks running concurrently don't step on each other.

Regarding traits + generics, I wanted to use them to interface the database API to mimic the dependency injection
paradigm. I actually did include this in an early draft but scrapped it to help simplify the development process given
the time constraints.

Lastly, I would have loved to implement Rust's type state pattern via `PhantomData`s to enfore the transaction
lifecycle. It's one of the reasons why Rust is so cool in that it is able to guarantee state machines are strictly
followed at compile time. But again, I was advised not to over-engineer this solution so I opted to not do this.

# AI Disclaimer

I used AI in 2 ways:

1. Google Search AI when looking up Rust documentation. Example query: "rust csv ignore whitespace"
2. ChatGPT to help generate 1 large test data set (10,000 rows). Prompt used:

```
generate me a transaction.csv file.

The headers: type,client,tx,amount

the valid types are: deposit, withdrawal, dispute, resolve, chargeback

client is of type u16

tx is of type u32

amount is any positive float. can have more than 4 decimal places

dispute, resolve, and chargeback types do not include an amount.
dispute, resolve, and chargeback types reference any previous row's client and tx as its own.
valid rows for dispute, resolve, and chargeback should have a trailing comma to indicate no amount.

generate 10,000 rows with random data that fit the criteria above

Can occasionally include invalid rows like ones with extra data or missing data. add less than 500 of these kinds of rows
```

No other AI was used for this project. All Rust code was handwritten. All project scoping/brainstorming was done by
yours truely.

# Assumptions

## Assumption 1
For any given client, the first transaction must be a `DEPOSIT`. Without this first `DEPOSIT` transaction, no other
transaction types would logically make sense. Perhaps `WITHDRAWAL` as a first transaction could work if the system
allows for negative account balances (i.e. a loan), but that does not seem to be the case based on the
specifications.

Do note that `DEPOSIT` as the first valid transaction is implicitly enforced, not explicitly.

## Assumption 2
A `RESOLVE` or `CHARGEBACK` transaction cannot occur before a `DISPUTE` transaction. That `DISPUTE` transaction must have a
matching "disputed" transaction, otherwise the `RESOLVE` or `CHARAGEBACK` transaction will be ignored.

## Assumption 3
A `DISPUTE` transaction must have an initual "disputed" transaction of type `DEPOSIT`. I believe this is implicitly implied in the
specifications ("... available funds should *decrease* by the amount disputed...").

There could be an argument to allow disputing withdrawals for cases where the account is compromised and the account
holder did not initiate the withdrawal. For this implementation, I will defer to the functionality described in the
specifications.

## Assumption 4
There can be multiple `DISPUTE` transactions "open" (i.e. without a resolution) at any given time.

## Assumption 5
As soon as an open `DISPUTE` transaction move into `CHARGEBACK` state, the account will be permanently locked (frozen),
even if there are other open `DISPUTE` transactions. No subsequent transactions will be applied to a locked account.

The specifications does not specify a way to unlock accounts. In a real world scenario, I imagine an account can be
unlocked through several ways, primarily asynchronously via emailing/calling the payment system's support team.

There could be an argument to allow `DEPOSIT` transactions to potentially settle any negative balances due to the chargeback.
The system could also allow open `DISPUTE` transactions to move into `RESOLVE` or also `CHARGEBACK` (if there are enough
available funds to issue the chargeback).

For this implementation, I will again defer to the functionality described (or in this case, omitted) in the specifications and
assume that locked accounts can never become unlocked. New transactions on the account will be ignored.

## Assumption 6

Each individual `amount` value and the accumulated `amount` value for each client will fit inside a `rust_decimal::Decimal`.
From the [documentation](https://docs.rs/rust_decimal/latest/rust_decimal/struct.Decimal.html#impl-Decimal):

```rust
assert_eq!(Decimal::MIN, dec!(-79_228_162_514_264_337_593_543_950_335));
assert_eq!(Decimal::MAX, dec!(79_228_162_514_264_337_593_543_950_335));
```

## Assumption 7

If there are multiple transactions with the same transaction id, then only the first transaction is applied. All
subsequent transactions with the same id will be seen as errors. Example:

```csv
type,client,tx,amount
deposit,1,1,1.0        <-- tx: 1, Applied
deposit,1,1,2.0        <-- tx: 1, Error (ignored)
withdrawal,1,1,5.0     <-- tx: 1, Error (ignored)
```

## Assumption 8

A new client should not be recorded to the database if the first transaction is not a `DEPOSIT`. For instance:

```csv
type,client,tx,amount
withdrawal,1,1,1.0
```

Should result in:

```csv
client,available,held,total,locked
```

This behavior could be argued either way (to record or not to record). I chose not to record because it could protect
the database against malicious actors who intentionally send bogus transactions. Plus, a client being absent from the
database implies that its record would be `<client_id>,0,0,0,false`.

## Assumption 9

If a transaction was disputed and then resolved, it can be disputed again in the future. Subsequent disputes can happen
in real life, particularly in situations where new information is discovered that could help support the dispute.

The payment system might want to cap the number of times a transaction can be disputed, but this implementation does not
enforce such limits.

## Assumption 10

A `DISPUTE`, `RESOLVE`, or `CHARGEBACK` transaction must have a matching ClientId as the initially disputed transaction.

## Assumption 11

Regarding how numbers are rounded: from my research, it seems like payment systems commonly use a rounding strategy
called "Banker's Rounding", which is different from the strategy typically taught at school. Apparently there are nice
statistical properties with using the former strategy, so I am inclined to use it for this implementation. Example:

```
5.4 -> 5
5.5 -> 6
5.6 -> 6

6.4 -> 6
6.5 -> 6 (not 7!)
6.6 -> 7
```

## Assumption 12

This implementation requires 2 key/value databases and currently implements them as BTree maps. In a real payment
system, these databases are almost certainly going to be remote (i.e. not stored on the server). But since that is not
the case for this implementation, I am going to assume that the input data set is not unreasonably large and/or the
hardware running this code will have enough memory to hold (worst case) roughly 2x the number of input transactions
(multiplied by the size of the Account and Transaction records).

The first database stores the Account summary information for each Client, allowing for dynamic updates. Recording this
information is unavoidable (in this scenario).

The second database stores the Transactions provided by input data set. Remembering which transactions happen is
important to the functionality of the payment system since it needs to prevent things like applying duplicate
transactions multiple times. There are a couple things I could implement to reduce the number of records stored:

1. Remove any transactions in stored with type `CHARGEBACK`. Since this is a terminal state, no future transactions with
   the same TransactionId will need to be applied
2. Remove all transactions for ClientIds whose accounts are locked. In this implementation, I assume that locked
   accounts can never become unlocked. Therefore, I could remove all transactions related to locked Clients to save
   memory space

## Assumption 13

The transaction type provided by the input CSV is all lowercase as shown in the specifications. Correctly spelled types
that are incorrectly cased will be ignored.
