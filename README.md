# Build And Run Instructions

```shell
cargo build

cargo run -- transactions.csv
```

Use the `-d` / `--debug` flag to see debug logs:

```shell
cargo run -- transactions.csv --debug
```

# Assumptions

## Assumption 1
For any given client, the first transaction must be a `DEPOSIT`. Without this first `DEPOSIT` transaction, no other
transaction types would logically make sense. Perhaps `WITHDRAWAL` as a first transaction could work if the system
allows for negative account balances (i.e. a loan), but that does not seem to be the case based on the
specifications.

Do note that `DEPOSIT` as the first valid transaction is implicitly enforced, not explicitly.

## Assumption 2
A `RESOLVE` or `CHARGEBACK` transaction cannot occur before a `DISPUTE` transaction. That `DISPUTE` transaction must have a
matching target transaction, otherwise the `RESOLVE` or `CHARAGEBACK` transaction will be ignored.

## Assumption 3
A `DISPUTE` transaction must have a target transaction of type `DEPOSIT`. I believe this is implicitly implied in the
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
