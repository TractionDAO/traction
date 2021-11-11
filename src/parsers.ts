import { Coder } from "@project-serum/anchor";
import type { KeyedAccountInfo } from "@solana/web3.js";

import type { OptionsContractData } from ".";
import { TractionJSON } from "./idls/traction";

/**
 * Coder for Traction data.
 */
export const TRACTION_CODER = new Coder(TractionJSON);

/**
 * Parser for use in Sail.
 * @param data
 * @returns
 */
export const optionsContractParser = (
  data: KeyedAccountInfo
): OptionsContractData => parseOptionsContract(data.accountInfo.data);

/**
 * Parses the options contract.
 * @param data
 * @returns
 */
export const parseOptionsContract = (data: Buffer): OptionsContractData =>
  TRACTION_CODER.accounts.decode<OptionsContractData>("OptionsContract", data);
