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
  getI64Decoder,
  getI64Encoder,
  getStructDecoder,
  getStructEncoder,
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
import {
  getNullableU64Decoder,
  getNullableU64Encoder,
  type NullableU64,
  type NullableU64Args,
} from '../../hooked';
import {
  getGovernanceConfigDecoder,
  getGovernanceConfigEncoder,
  getProposalStatusDecoder,
  getProposalStatusEncoder,
  type GovernanceConfig,
  type GovernanceConfigArgs,
  type ProposalStatus,
  type ProposalStatusArgs,
} from '../types';

export type Proposal = {
  discriminator: Array<number>;
  author: Address;
  cooldownTimestamp: NullableU64;
  creationTimestamp: bigint;
  governanceConfig: GovernanceConfig;
  stakeAbstained: bigint;
  stakeAgainst: bigint;
  stakeFor: bigint;
  status: ProposalStatus;
  padding: Array<number>;
  votingStartTimestamp: NullableU64;
};

export type ProposalArgs = {
  discriminator: Array<number>;
  author: Address;
  cooldownTimestamp: NullableU64Args;
  creationTimestamp: number | bigint;
  governanceConfig: GovernanceConfigArgs;
  stakeAbstained: number | bigint;
  stakeAgainst: number | bigint;
  stakeFor: number | bigint;
  status: ProposalStatusArgs;
  padding: Array<number>;
  votingStartTimestamp: NullableU64Args;
};

export function getProposalEncoder(): Encoder<ProposalArgs> {
  return getStructEncoder([
    ['discriminator', getArrayEncoder(getU8Encoder(), { size: 8 })],
    ['author', getAddressEncoder()],
    ['cooldownTimestamp', getNullableU64Encoder()],
    ['creationTimestamp', getI64Encoder()],
    ['governanceConfig', getGovernanceConfigEncoder()],
    ['stakeAbstained', getU64Encoder()],
    ['stakeAgainst', getU64Encoder()],
    ['stakeFor', getU64Encoder()],
    ['status', getProposalStatusEncoder()],
    ['padding', getArrayEncoder(getU8Encoder(), { size: 7 })],
    ['votingStartTimestamp', getNullableU64Encoder()],
  ]);
}

export function getProposalDecoder(): Decoder<Proposal> {
  return getStructDecoder([
    ['discriminator', getArrayDecoder(getU8Decoder(), { size: 8 })],
    ['author', getAddressDecoder()],
    ['cooldownTimestamp', getNullableU64Decoder()],
    ['creationTimestamp', getI64Decoder()],
    ['governanceConfig', getGovernanceConfigDecoder()],
    ['stakeAbstained', getU64Decoder()],
    ['stakeAgainst', getU64Decoder()],
    ['stakeFor', getU64Decoder()],
    ['status', getProposalStatusDecoder()],
    ['padding', getArrayDecoder(getU8Decoder(), { size: 7 })],
    ['votingStartTimestamp', getNullableU64Decoder()],
  ]);
}

export function getProposalCodec(): Codec<ProposalArgs, Proposal> {
  return combineCodec(getProposalEncoder(), getProposalDecoder());
}

export function decodeProposal<TAddress extends string = string>(
  encodedAccount: EncodedAccount<TAddress>
): Account<Proposal, TAddress>;
export function decodeProposal<TAddress extends string = string>(
  encodedAccount: MaybeEncodedAccount<TAddress>
): MaybeAccount<Proposal, TAddress>;
export function decodeProposal<TAddress extends string = string>(
  encodedAccount: EncodedAccount<TAddress> | MaybeEncodedAccount<TAddress>
): Account<Proposal, TAddress> | MaybeAccount<Proposal, TAddress> {
  return decodeAccount(
    encodedAccount as MaybeEncodedAccount<TAddress>,
    getProposalDecoder()
  );
}

export async function fetchProposal<TAddress extends string = string>(
  rpc: Parameters<typeof fetchEncodedAccount>[0],
  address: Address<TAddress>,
  config?: FetchAccountConfig
): Promise<Account<Proposal, TAddress>> {
  const maybeAccount = await fetchMaybeProposal(rpc, address, config);
  assertAccountExists(maybeAccount);
  return maybeAccount;
}

export async function fetchMaybeProposal<TAddress extends string = string>(
  rpc: Parameters<typeof fetchEncodedAccount>[0],
  address: Address<TAddress>,
  config?: FetchAccountConfig
): Promise<MaybeAccount<Proposal, TAddress>> {
  const maybeAccount = await fetchEncodedAccount(rpc, address, config);
  return decodeProposal(maybeAccount);
}

export async function fetchAllProposal(
  rpc: Parameters<typeof fetchEncodedAccounts>[0],
  addresses: Array<Address>,
  config?: FetchAccountsConfig
): Promise<Account<Proposal>[]> {
  const maybeAccounts = await fetchAllMaybeProposal(rpc, addresses, config);
  assertAccountsExist(maybeAccounts);
  return maybeAccounts;
}

export async function fetchAllMaybeProposal(
  rpc: Parameters<typeof fetchEncodedAccounts>[0],
  addresses: Array<Address>,
  config?: FetchAccountsConfig
): Promise<MaybeAccount<Proposal>[]> {
  const maybeAccounts = await fetchEncodedAccounts(rpc, addresses, config);
  return maybeAccounts.map((maybeAccount) => decodeProposal(maybeAccount));
}
