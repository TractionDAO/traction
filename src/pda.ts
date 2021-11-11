import { utils } from "@project-serum/anchor";
import { u64 } from "@saberhq/token-utils";
import { PublicKey } from "@solana/web3.js";

import { TRACTION_ADDRESSES } from "./constants";

/**
 * Finds the address of the options contract.
 * @returns
 */
export const findOptionsContractAddress = async ({
  programId = TRACTION_ADDRESSES.Traction,
  underlyingMint,
  quoteMint,
  strike,
  expiryTs,
  isPut,
}: {
  programId?: PublicKey;
  underlyingMint: PublicKey;
  quoteMint: PublicKey;
  strike: u64;
  expiryTs: number;
  isPut: boolean;
}): Promise<[PublicKey, number]> => {
  return await PublicKey.findProgramAddress(
    [
      utils.bytes.utf8.encode("OptionsContract"),
      underlyingMint.toBuffer(),
      quoteMint.toBuffer(),
      strike.toBuffer(),
      new u64(expiryTs).toBuffer(),
      Buffer.from([isPut ? 1 : 0]),
    ],
    programId
  );
};
