/**
 * This code was AUTOGENERATED using the kinobi library.
 * Please DO NOT EDIT THIS FILE, instead use visitors
 * to add features, then rerun kinobi to update it.
 *
 * @see https://github.com/kinobi-so/kinobi
 */

import {
  assertAccountExists,
  assertAccountsExist,
  combineCodec,
  decodeAccount,
  fetchEncodedAccount,
  fetchEncodedAccounts,
  getAddressDecoder,
  getAddressEncoder,
  getArrayDecoder,
  getArrayEncoder,
  getStructDecoder,
  getStructEncoder,
  getU32Decoder,
  getU32Encoder,
  getU64Decoder,
  getU64Encoder,
  getU8Decoder,
  getU8Encoder,
  type Account,
  type Address,
  type Codec,
  type Decoder,
  type EncodedAccount,
  type Encoder,
  type FetchAccountConfig,
  type FetchAccountsConfig,
  type MaybeAccount,
  type MaybeEncodedAccount,
} from '@solana/web3.js';

export type GovernanceConfig = {
  cooldownPeriodSeconds: bigint;
  proposalAcceptanceThreshold: number;
  proposalRejectionThreshold: number;
  signerBumpSeed: number;
  padding: Array<number>;
  stakeConfigAddress: Address;
  votingPeriodSeconds: bigint;
};

export type GovernanceConfigArgs = {
  cooldownPeriodSeconds: number | bigint;
  proposalAcceptanceThreshold: number;
  proposalRejectionThreshold: number;
  signerBumpSeed: number;
  padding: Array<number>;
  stakeConfigAddress: Address;
  votingPeriodSeconds: number | bigint;
};

export function getGovernanceConfigEncoder(): Encoder<GovernanceConfigArgs> {
  return getStructEncoder([
    ['cooldownPeriodSeconds', getU64Encoder()],
    ['proposalAcceptanceThreshold', getU32Encoder()],
    ['proposalRejectionThreshold', getU32Encoder()],
    ['signerBumpSeed', getU8Encoder()],
    ['padding', getArrayEncoder(getU8Encoder(), { size: 7 })],
    ['stakeConfigAddress', getAddressEncoder()],
    ['votingPeriodSeconds', getU64Encoder()],
  ]);
}

export function getGovernanceConfigDecoder(): Decoder<GovernanceConfig> {
  return getStructDecoder([
    ['cooldownPeriodSeconds', getU64Decoder()],
    ['proposalAcceptanceThreshold', getU32Decoder()],
    ['proposalRejectionThreshold', getU32Decoder()],
    ['signerBumpSeed', getU8Decoder()],
    ['padding', getArrayDecoder(getU8Decoder(), { size: 7 })],
    ['stakeConfigAddress', getAddressDecoder()],
    ['votingPeriodSeconds', getU64Decoder()],
  ]);
}

export function getGovernanceConfigCodec(): Codec<
  GovernanceConfigArgs,
  GovernanceConfig
> {
  return combineCodec(
    getGovernanceConfigEncoder(),
    getGovernanceConfigDecoder()
  );
}

export function decodeGovernanceConfig<TAddress extends string = string>(
  encodedAccount: EncodedAccount<TAddress>
): Account<GovernanceConfig, TAddress>;
export function decodeGovernanceConfig<TAddress extends string = string>(
  encodedAccount: MaybeEncodedAccount<TAddress>
): MaybeAccount<GovernanceConfig, TAddress>;
export function decodeGovernanceConfig<TAddress extends string = string>(
  encodedAccount: EncodedAccount<TAddress> | MaybeEncodedAccount<TAddress>
):
  | Account<GovernanceConfig, TAddress>
  | MaybeAccount<GovernanceConfig, TAddress> {
  return decodeAccount(
    encodedAccount as MaybeEncodedAccount<TAddress>,
    getGovernanceConfigDecoder()
  );
}

export async function fetchGovernanceConfig<TAddress extends string = string>(
  rpc: Parameters<typeof fetchEncodedAccount>[0],
  address: Address<TAddress>,
  config?: FetchAccountConfig
): Promise<Account<GovernanceConfig, TAddress>> {
  const maybeAccount = await fetchMaybeGovernanceConfig(rpc, address, config);
  assertAccountExists(maybeAccount);
  return maybeAccount;
}

export async function fetchMaybeGovernanceConfig<
  TAddress extends string = string,
>(
  rpc: Parameters<typeof fetchEncodedAccount>[0],
  address: Address<TAddress>,
  config?: FetchAccountConfig
): Promise<MaybeAccount<GovernanceConfig, TAddress>> {
  const maybeAccount = await fetchEncodedAccount(rpc, address, config);
  return decodeGovernanceConfig(maybeAccount);
}

export async function fetchAllGovernanceConfig(
  rpc: Parameters<typeof fetchEncodedAccounts>[0],
  addresses: Array<Address>,
  config?: FetchAccountsConfig
): Promise<Account<GovernanceConfig>[]> {
  const maybeAccounts = await fetchAllMaybeGovernanceConfig(
    rpc,
    addresses,
    config
  );
  assertAccountsExist(maybeAccounts);
  return maybeAccounts;
}

export async function fetchAllMaybeGovernanceConfig(
  rpc: Parameters<typeof fetchEncodedAccounts>[0],
  addresses: Array<Address>,
  config?: FetchAccountsConfig
): Promise<MaybeAccount<GovernanceConfig>[]> {
  const maybeAccounts = await fetchEncodedAccounts(rpc, addresses, config);
  return maybeAccounts.map((maybeAccount) =>
    decodeGovernanceConfig(maybeAccount)
  );
}

export function getGovernanceConfigSize(): number {
  return 64;
}
