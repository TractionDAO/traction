import {
  CRATE_ADDRESSES,
  generateCrateAddress,
} from "@crateprotocol/crate-sdk";
import { Program, Provider as AnchorProvider } from "@project-serum/anchor";
import type { Provider } from "@saberhq/solana-contrib";
import {
  SignerWallet,
  SolanaProvider,
  TransactionEnvelope,
} from "@saberhq/solana-contrib";
import type { Price } from "@saberhq/token-utils";
import {
  createInitMintInstructions,
  getOrCreateATAs,
  TokenAmount,
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
    public readonly provider: Provider,
    public readonly programs: TractionPrograms
  ) {}

  /**
   * Initialize from a Provider
   * @param provider
   * @returns
   */
  static init(provider: Provider): TractionSDK {
    const anchorProvider = new AnchorProvider(
      provider.connection,
      provider.wallet,
      provider.opts
    );
    return new TractionSDK(provider, {
      Traction: new Program(
        TractionJSON,
        TRACTION_ADDRESSES.Traction,
        anchorProvider
      ) as unknown as TractionProgram,
    });
  }

  /**
   * Creates a new instance of the SDK with the given keypair.
   */
  public withSigner(signer: Signer): TractionSDK {
    return TractionSDK.init(
      new SolanaProvider(
        this.provider.connection,
        this.provider.broadcaster,
        new SignerWallet(signer),
        this.provider.opts
      )
    );
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
      decimals: optionsContract.decimals,
      mintAuthority: contractKey,
      freezeAuthority: contractKey,
    });

    // normalize strike to 10*9 of underlying
    const strikeNormalized = strike.quote(new TokenAmount(underlying, 10 ** 9));

    const newContractIx = this.programs.Traction.instruction.newContract(
      strikeNormalized.toU64(),
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
        quote: optionsContract.quote.mintAccount,
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
