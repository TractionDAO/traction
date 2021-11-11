# traction

[![License](https://img.shields.io/crates/l/traction)](https://github.com/TractionDAO/traction/blob/master/LICENSE.md)
[![Build Status](https://img.shields.io/github/workflow/status/TractionDAO/traction/E2E/master)](https://github.com/TractionDAO/traction/actions/workflows/programs-e2e.yml?query=branch%3Amaster)
[![Contributors](https://img.shields.io/github/contributors/TractionDAO/traction)](https://github.com/TractionDAO/traction/graphs/contributors)
[![Chat](https://img.shields.io/badge/chat-on%20keybase-success)](https://keybase.io/team/TractionDAO)

Traction is a protocol for issuing American options on Solana.

Follow us for updates below:

- Twitter: https://twitter.com/TractionDAO
- Keybase: https://keybase.io/team/TractionDAO

## About

Traction is a Solana protocol which handles the lifecycle of American options. There are five actions one can take:

- `new_contract`: Creates a new options market associated with an underlying, a quote asset, a strike, a direction (put or call), and an expiry.
- `write`: Issues an option, with the underlying held as collateral.
- `exercise`: Exchangs quote tokens for underlying tokens at the strike price.
- `redeem`: When the option has passed expiry, this allows an option writer to retrieve their underlying collateral.
- _(unimplemented)_ `exit`: If the option has yet to expire, this allows an option writer to retrieve their collateral by buying an option off the open market.

## Packages

| Package                 | Description                           | Version                                                                                                               | Docs                                                                                        |
| :---------------------- | :------------------------------------ | :-------------------------------------------------------------------------------------------------------------------- | :------------------------------------------------------------------------------------------ |
| `traction`              | Program for issuing American options. | [![Crates.io](https://img.shields.io/crates/v/traction)](https://crates.io/crates/traction)                           | [![Docs.rs](https://docs.rs/traction/badge.svg)](https://docs.rs/traction)                  |
| `@tractiondao/traction` | TypeScript SDK for Traction           | [![npm](https://img.shields.io/npm/v/@tractiondao/traction.svg)](https://www.npmjs.com/package/@tractiondao/traction) | [![Docs](https://img.shields.io/badge/docs-typedoc-blue)](https://docs.traction.market/ts/) |

## Note

- **Traction is in active development, so all APIs are subject to change.**
- **This code is unaudited. Use at your own risk.**

## Contribution

Thank you for your interest in contributing to Traction Protocol! All contributions are welcome no
matter how big or small. This includes (but is not limited to) filing issues,
adding documentation, fixing bugs, creating examples, and implementing features.

If you'd like to contribute, please claim an issue by commenting, forking, and
opening a pull request, even if empty. This allows the maintainers to track who
is working on what issue as to not overlap work.

For simple documentation changes, feel free to just open a pull request.

If you're considering larger changes or self motivated features, please file an issue
and engage with the maintainers by joining the development channel on [Keybase](https://keybase.io/team/TractionDAO).

## License

Traction Protocol is licensed under [the Affero GPL 3.0 license](/LICENSE.txt).

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in Traction Protocol by you, as defined in the AGPL-3.0 license, shall be licensed as above, without any additional terms or conditions.
