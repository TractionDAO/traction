import { CRATE_ADDRESSES } from "@crateprotocol/crate-sdk";
import type { Provider } from "@saberhq/solana-contrib";
import { TransactionEnvelope } from "@saberhq/solana-contrib";
import type { NumberFormat, Price, TokenInfo, u64 } from "@saberhq/token-utils";
import {
  getATAAddress,
  getOrCreateATAs,
  Token,
  TOKEN_PROGRAM_ID,
  TokenAmount,
} from "@saberhq/token-utils";
import type { PublicKey } from "@solana/web3.js";

import { FEE_OWNER } from "./constants";
import { findOptionsContractAddress } from "./pda";
import type { OptionsContractData, TractionProgram } from "./programs/traction";
import type { TractionSDK } from "./traction";

/**
 * Wrapper for interacting with an options contract.
 */
export class OptionsContract {
  private _data: OptionsContractData | null = null;

  constructor(
    readonly sdk: TractionSDK,
    readonly strike: Price,
    readonly expiryTs: number,
    readonly isPut: boolean
  ) {}

  get program(): TractionProgram {
    return this.sdk.programs.Traction;
  }

  get provider(): Provider {
    return this.sdk.provider;
  }

  /**
   * The underlying {@link Token}.
   */
  get underlying(): Token {
    return this.strike.baseCurrency;
  }

  /**
   * The quote {@link Token}.
   */
  get quote(): Token {
    return this.strike.quoteCurrency;
  }

  /**
   * Amount of strike tokens for `10**9` of the underlying.
   * This is used to compute the PDA.
   */
  get rawStrike(): u64 {
    return this.strike.quote(new TokenAmount(this.underlying, 10 ** 9)).toU64();
  }

  /**
   * Amount of strike tokens for 1 unit of the underlying.
   */
  get strikeQuoteForUnderlying(): TokenAmount {
    return this.strike.quote(
      new TokenAmount(this.underlying, 10 ** this.underlying.decimals)
    );
  }

  /**
   * Amount of underlying tokens for 1 unit of the quote.
   */
  get strikeUnderlyingForQuote(): TokenAmount {
    return this.strike
      .invert()
      .quote(new TokenAmount(this.quote, 10 ** this.quote.decimals));
  }

  /**
   * Expiry date.
   */
  get expiry(): Date {
    return new Date(this.expiryTs * 1_000);
  }

  /**
   * Human-readable expiry of the option.
   */
  get formattedExpiry(): string {
    const expiry = this.expiry;
    if (expiry.getFullYear() === new Date().getFullYear()) {
      return expiry.toLocaleDateString(undefined, {
        month: "short",
        day: "numeric",
      });
    }
    return expiry.toLocaleDateString(undefined, {
      month: "short",
      day: "numeric",
      year: "numeric",
    });
  }

  /**
   * Human-readable expiry of the option. Meant to be used in the symbol.
   */
  get formattedExpiryShort(): string {
    const expiry = this.expiry;
    const monthStr = expiry
      .toLocaleDateString(undefined, { month: "short" })
      .toUpperCase();
    const day = expiry.getDate();
    const yearsMatch = expiry.getFullYear() === new Date().getFullYear();

    // only show day if it's not the 1st
    // only show year if it's not the current year
    return `${day !== 1 ? day : ""}${monthStr}${
      yearsMatch ? "" : `${expiry.getFullYear()}`
    }`;
  }

  get renderedStrike(): TokenAmount {
    return this.strikeUnderlyingForQuote;
  }

  /**
   * Formats the strike.
   * @param format
   * @returns
   */
  formatStrike(format?: NumberFormat | undefined): string {
    return this.renderedStrike.formatUnits(format);
  }

  /**
   * Decimals that this option should have.
   */
  get decimals(): number {
    return this.underlying.decimals;
  }

  /**
   * Token symbol.
   */
  get symbol(): string {
    const [underlying, quote] = this.isPut
      ? ([this.quote, this.underlying] as const)
      : ([this.underlying, this.quote] as const);
    return `${this.formattedExpiryShort}-${underlying.symbol}-${
      this.isPut ? "P" : "C"
    }${this.renderedStrike.toExact()}-${quote.symbol}`;
  }

  /**
   * Token name.
   */
  get name(): string {
    return `${this.formattedExpiry} ${
      this.isPut ? this.quote.symbol : this.underlying.symbol
    } ${this.formatStrike()} ${this.isPut ? " PUT" : " CALL"}`;
  }

  /**
   * Generates a human-friendly {@link TokenInfo} of the options contract.
   *
   * @param isPut
   * @returns
   */
  async generateTokenInfo(): Promise<TokenInfo> {
    const [address] = await this.findAddress();
    return {
      chainId: this.underlying.chainId,
      address: address.toString(),
      name: this.name,
      symbol: this.symbol,
      decimals: this.decimals,
      extensions: {
        source: "traction",
        website: `https://traction.market/#/option/${address.toString()}`,
      },
    };
  }

  /**
   * The writer token of the option.
   */
  async fetchWriterToken(): Promise<Token> {
    return new Token({
      chainId: this.underlying.chainId,
      address: (await this.fetch()).data.writerMint.toString(),
      name: `${this.name} Writer`,
      symbol: `wrt${this.symbol}`,
      decimals: this.decimals,
    });
  }

  /**
   * Generates the Option {@link Token} from a {@link TokenInfo}.
   * @returns
   */
  async generateToken(): Promise<Token> {
    return new Token(await this.generateTokenInfo());
  }

  /**
   * Finds the address of this {@link OptionsContract}.
   * @returns
   */
  async findAddress(): Promise<[PublicKey, number]> {
    return await findOptionsContractAddress({
      underlyingMint: this.underlying.mintAccount,
      quoteMint: this.strike.quoteCurrency.mintAccount,
      strike: this.rawStrike,
      expiryTs: this.expiryTs,
      isPut: this.isPut,
    });
  }

  /**
   * Writes an option
   * @returns
   */
  async write({
    writerAuthority = this.provider.wallet.publicKey,
    writeAmount,
  }: {
    writerAuthority?: PublicKey;
    writeAmount: TokenAmount;
  }): Promise<TransactionEnvelope> {
    const { key: contract, data: contractData } = await this.fetch();

    const writerATAs = await getOrCreateATAs({
      provider: this.provider,
      mints: {
        collateral: this.collateralToken.mintAccount,
        writer: contractData.writerMint,
        option: contractData.optionMint,
      },
    });
    const crateATAs = await getOrCreateATAs({
      provider: this.provider,
      mints: {
        collateral: this.collateralToken.mintAccount,
      },
      owner: contractData.writerCrate,
    });

    const writeIX = this.program.instruction.optionWrite(writeAmount.toU64(), {
      accounts: {
        writerAuthority,
        contract,

        userCollateralFundingTokens: writerATAs.accounts.collateral,
        writerTokenDestination: writerATAs.accounts.writer,
        optionTokenDestination: writerATAs.accounts.option,
        crateCollateralTokens: crateATAs.accounts.collateral,
        writerMint: contractData.writerMint,
        optionMint: contractData.optionMint,

        writerCrateToken: contractData.writerCrate,
        tokenProgram: TOKEN_PROGRAM_ID,
        crateTokenProgram: CRATE_ADDRESSES.CrateToken,
      },
    });

    return new TransactionEnvelope(this.provider, [
      ...writerATAs.instructions,
      ...crateATAs.instructions,
      writeIX,
    ]);
  }

  /**
   * Exercises an option
   * @returns
   */
  async exercise({
    exerciserAuthority = this.provider.wallet.publicKey,
    optionAmount,
  }: {
    exerciserAuthority?: PublicKey;
    optionAmount: TokenAmount;
  }): Promise<TransactionEnvelope> {
    const { key: contract, data: contractData } = await this.fetch();

    const writerATAs = await getOrCreateATAs({
      provider: this.provider,
      mints: {
        collateral: this.collateralToken.mintAccount,
        option: contractData.optionMint,
        exercise: this.exerciseToken.mintAccount,
      },
    });
    const crateATAs = await getOrCreateATAs({
      provider: this.provider,
      mints: {
        collateral: this.collateralToken.mintAccount,
        exercise: this.exerciseToken.mintAccount,
      },
      owner: contractData.writerCrate,
    });
    const exerciseFeeDestination = await getATAAddress({
      mint: this.exerciseToken.mintAccount,
      owner: FEE_OWNER,
    });

    const exerciseIX = this.program.instruction.optionExercise(
      optionAmount.toU64(),
      {
        accounts: {
          exerciserAuthority,
          contract,

          exerciseTokenSource: writerATAs.accounts.exercise,
          optionMint: contractData.optionMint,
          optionTokenSource: writerATAs.accounts.option,
          writerCrateToken: contractData.writerCrate,
          crateCollateralTokens: crateATAs.accounts.collateral,
          crateExerciseTokens: crateATAs.accounts.exercise,
          collateralTokenDestination: writerATAs.accounts.collateral,
          exerciseFeeDestination,

          tokenProgram: TOKEN_PROGRAM_ID,
          crateTokenProgram: CRATE_ADDRESSES.CrateToken,
        },
      }
    );

    return new TransactionEnvelope(this.provider, [
      ...writerATAs.instructions,
      ...crateATAs.instructions,
      exerciseIX,
    ]);
  }

  /**
   * Fetches the data associated with the contract.
   */
  async fetch(): Promise<{ key: PublicKey; data: OptionsContractData }> {
    const [contract] = await this.findAddress();
    if (this._data) {
      return { key: contract, data: this._data };
    }
    const contractData =
      (await this.program.account.optionsContract.fetchNullable(
        contract
      )) as OptionsContractData;
    if (!contractData) {
      throw new Error(
        `could not fetch OptionsContract at ${contract.toString()}`
      );
    }
    this._data = contractData;
    return { key: contract, data: contractData };
  }

  /**
   * Collateral token.
   */
  get collateralToken(): Token {
    return this.isPut ? this.quote : this.underlying;
  }

  /**
   * Exercise token.
   */
  get exerciseToken(): Token {
    return this.isPut ? this.underlying : this.quote;
  }

  /**
   * Redeems option writer tokens for the underlying.
   * @returns
   */
  async redeem({
    writerAuthority = this.provider.wallet.publicKey,
    writerAmount,
  }: {
    writerAuthority?: PublicKey;
    writerAmount: TokenAmount;
  }): Promise<TransactionEnvelope> {
    const { key: contract, data: contractData } = await this.fetch();

    const writerATAs = await getOrCreateATAs({
      provider: this.provider,
      mints: {
        underlying: this.underlying.mintAccount,
        writer: contractData.writerMint,
        quote: this.quote.mintAccount,
      },
    });
    const crateATAs = await getOrCreateATAs({
      provider: this.provider,
      mints: {
        collateral: this.collateralToken.mintAccount,
        exercise: this.exerciseToken.mintAccount,
      },
      owner: contractData.writerCrate,
    });

    const redeemIX = this.program.instruction.optionRedeem(
      writerAmount.toU64(),
      {
        accounts: {
          writerAuthority,
          contract,

          writerTokenSource: writerATAs.accounts.writer,
          writerMint: contractData.writerMint,
          crateCollateralTokens: crateATAs.accounts.collateral,
          crateExerciseTokens: crateATAs.accounts.exercise,
          underlyingTokenDestination: writerATAs.accounts.underlying,
          quoteTokenDestination: writerATAs.accounts.quote,

          tokenProgram: TOKEN_PROGRAM_ID,
          writerCrateToken: contractData.writerCrate,
          crateTokenProgram: CRATE_ADDRESSES.CrateToken,
        },
      }
    );

    return new TransactionEnvelope(this.provider, [
      ...writerATAs.instructions,
      ...crateATAs.instructions,
      redeemIX,
    ]);
  }
}
