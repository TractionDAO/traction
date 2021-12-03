import {
  CRATE_ADDRESSES,
  generateCrateAddress,
} from "@crateprotocol/crate-sdk";
import { newProgramMap } from "@saberhq/anchor-contrib";
import type { AugmentedProvider, Provider } from "@saberhq/solana-contrib";
import {
  SolanaAugmentedProvider,
  TransactionEnvelope,
} from "@saberhq/solana-contrib";
import {
  createInitMintInstructions,
  deserializeMint,
  getOrCreateATAs,
  Price,
  Token,
  u64,
} from "@saberhq/token-utils";
import type { PublicKey, Signer } from "@solana/web3.js";
import { Keypair, SystemProgram } from "@solana/web3.js";

import { FEE_OWNER, TRACTION_ADDRESSES } from "./constants";
import { TractionJSON } from "./idls/traction";
import { OptionsContract } from "./optionsContract";
import type { TractionProgram } from "./programs/traction";

/**
 * Programs associated with the Traction protocol.
 */
export interface TractionPrograms {
  Traction: TractionProgram;
}

/**
 * JavaScript SDK for interacting with Traction.
 */
export class TractionSDK {
  constructor(
    public readonly provider: AugmentedProvider,
    public readonly programs: TractionPrograms
  ) {}

  /**
   * Initialize from a Provider
   * @param provider
   * @returns
   */
  static init(provider: Provider): TractionSDK {
    return new TractionSDK(
      new SolanaAugmentedProvider(provider),
      newProgramMap<TractionPrograms>(
        provider,
        { Traction: TractionJSON },
        TRACTION_ADDRESSES
      )
    );
  }

  /**
   * Creates a new instance of the SDK with the given keypair.
   */
  withSigner(signer: Signer): TractionSDK {
    return TractionSDK.init(this.provider.withSigner(signer));
  }

  loadContract({
    strike,
    expiryTs,
    direction,
  }: {
    strike: Price;
    /**
     * Expiry timestamp, in seconds since epoch.
     */
    expiryTs: number;
    direction: "put" | "call";
  }): OptionsContract {
    const isPut = direction === "put";
    return new OptionsContract(this, strike, expiryTs, isPut);
  }

  async loadContractFromKey({
    key,
    underlying,
    quote,
  }: {
    key: PublicKey;
    underlying?: Token;
    quote?: Token;
  }): Promise<OptionsContract | null> {
    const contractData =
      await this.programs.Traction.account.optionsContract.fetchNullable(key);
    if (!contractData) {
      return null;
    }
    if (!underlying) {
      const underlyingMintRaw = await this.provider.getAccountInfo(
        contractData.underlyingMint
      );
      if (!underlyingMintRaw) {
        throw new Error(
          `Could not fetch underlying mint: ${contractData.underlyingMint.toString()}`
        );
      }
      const underlyingMintParsed = deserializeMint(
        underlyingMintRaw.accountInfo.data
      );
      underlying = Token.fromMint(
        contractData.underlyingMint,
        underlyingMintParsed.decimals
      );
    }
    if (!quote) {
      const quoteMintRaw = await this.provider.getAccountInfo(
        contractData.quoteMint
      );
      if (!quoteMintRaw) {
        throw new Error(
          `Could not fetch quote mint: ${contractData.quoteMint.toString()}`
        );
      }
      const quoteMintParsed = deserializeMint(quoteMintRaw.accountInfo.data);
      quote = Token.fromMint(contractData.quoteMint, quoteMintParsed.decimals);
    }
    const strike = new Price(underlying, quote, 10 ** 9, contractData.strike);
    return new OptionsContract(
      this,
      strike,
      contractData.expiryTs.toNumber(),
      !!contractData.isPut
    );
  }

  /**
   * Creates a new options contract.
   * @returns
   */
  async newContract({
    payer = this.provider.wallet.publicKey,
    writerMintKP = Keypair.generate(),
    optionMintKP = Keypair.generate(),
    strike,
    expiryTs,
    direction,
  }: {
    payer?: PublicKey;
    writerMintKP?: Keypair;
    optionMintKP?: Keypair;
    strike: Price;
    /**
     * Expiry timestamp, in seconds since epoch.
     */
    expiryTs: number;
    direction: "put" | "call";
  }): Promise<{
    optionsContract: OptionsContract;
    tx: TransactionEnvelope;
  }> {
    const isPut = direction === "put";
    const underlying = strike.baseCurrency;
    const optionsContract = new OptionsContract(this, strike, expiryTs, isPut);
    const [contractKey, contractBump] = await optionsContract.findAddress();
    const { instructions: createAccountInstructions } = await getOrCreateATAs({
      provider: this.provider,
      mints: {
        underlying: optionsContract.underlying.mintAccount,
        quote: optionsContract.quote.mintAccount,
      },
      owner: contractKey,
    });

    const [crateToken, crateBump] = await generateCrateAddress(
      writerMintKP.publicKey
    );
    const createWriterMint = await createInitMintInstructions({
      provider: this.provider,
      mintKP: writerMintKP,
      decimals: underlying.decimals,
      mintAuthority: crateToken,
      freezeAuthority: crateToken,
    });
    const createOptionMint = await createInitMintInstructions({
      provider: this.provider,
      mintKP: optionMintKP,
      decimals: underlying.decimals,
      mintAuthority: contractKey,
      freezeAuthority: contractKey,
    });

    const newContractIx = this.programs.Traction.instruction.newContract(
      optionsContract.rawStrike,
      new u64(expiryTs),
      isPut,
      contractBump,
      crateBump,
      {
        accounts: {
          contract: contractKey,

          underlyingMint: underlying.mintAccount,
          quoteMint: optionsContract.quote.mintAccount,
          optionMint: optionMintKP.publicKey,
          writerCrate: {
            crateMint: writerMintKP.publicKey,
            crateToken,
            crateTokenProgram: CRATE_ADDRESSES.CrateToken,
          },

          payer,
          systemProgram: SystemProgram.programId,
        },
      }
    );

    // create the exercise fee ATA in creation
    // so we don't have to keep fetching it later
    const feeATAs = await getOrCreateATAs({
      provider: this.provider,
      mints: {
        exercise: optionsContract.exerciseToken.mintAccount,
      },
      owner: FEE_OWNER,
    });

    const newContractTX = new TransactionEnvelope(
      this.provider,
      [
        ...createAccountInstructions,
        ...createWriterMint.instructions,
        ...createOptionMint.instructions,
        ...feeATAs.instructions,
        newContractIx,
      ],
      [writerMintKP, optionMintKP]
    );

    return {
      optionsContract,
      tx: newContractTX,
    };
  }
}
