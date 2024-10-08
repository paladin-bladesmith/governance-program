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
  getProposalVoteElectionDecoder,
  getProposalVoteElectionEncoder,
  type ProposalVoteElection,
  type ProposalVoteElectionArgs,
} from '../types';

export type ProposalVote = {
  proposalAddress: Address;
  stake: bigint;
  stakeAddress: Address;
  election: ProposalVoteElection;
  padding: Array<number>;
};

export type ProposalVoteArgs = {
  proposalAddress: Address;
  stake: number | bigint;
  stakeAddress: Address;
  election: ProposalVoteElectionArgs;
  padding: Array<number>;
};

export function getProposalVoteEncoder(): Encoder<ProposalVoteArgs> {
  return getStructEncoder([
    ['proposalAddress', getAddressEncoder()],
    ['stake', getU64Encoder()],
    ['stakeAddress', getAddressEncoder()],
    ['election', getProposalVoteElectionEncoder()],
    ['padding', getArrayEncoder(getU8Encoder(), { size: 7 })],
  ]);
}

export function getProposalVoteDecoder(): Decoder<ProposalVote> {
  return getStructDecoder([
    ['proposalAddress', getAddressDecoder()],
    ['stake', getU64Decoder()],
    ['stakeAddress', getAddressDecoder()],
    ['election', getProposalVoteElectionDecoder()],
    ['padding', getArrayDecoder(getU8Decoder(), { size: 7 })],
  ]);
}

export function getProposalVoteCodec(): Codec<ProposalVoteArgs, ProposalVote> {
  return combineCodec(getProposalVoteEncoder(), getProposalVoteDecoder());
}

export function decodeProposalVote<TAddress extends string = string>(
  encodedAccount: EncodedAccount<TAddress>
): Account<ProposalVote, TAddress>;
export function decodeProposalVote<TAddress extends string = string>(
  encodedAccount: MaybeEncodedAccount<TAddress>
): MaybeAccount<ProposalVote, TAddress>;
export function decodeProposalVote<TAddress extends string = string>(
  encodedAccount: EncodedAccount<TAddress> | MaybeEncodedAccount<TAddress>
): Account<ProposalVote, TAddress> | MaybeAccount<ProposalVote, TAddress> {
  return decodeAccount(
    encodedAccount as MaybeEncodedAccount<TAddress>,
    getProposalVoteDecoder()
  );
}

export async function fetchProposalVote<TAddress extends string = string>(
  rpc: Parameters<typeof fetchEncodedAccount>[0],
  address: Address<TAddress>,
  config?: FetchAccountConfig
): Promise<Account<ProposalVote, TAddress>> {
  const maybeAccount = await fetchMaybeProposalVote(rpc, address, config);
  assertAccountExists(maybeAccount);
  return maybeAccount;
}

export async function fetchMaybeProposalVote<TAddress extends string = string>(
  rpc: Parameters<typeof fetchEncodedAccount>[0],
  address: Address<TAddress>,
  config?: FetchAccountConfig
): Promise<MaybeAccount<ProposalVote, TAddress>> {
  const maybeAccount = await fetchEncodedAccount(rpc, address, config);
  return decodeProposalVote(maybeAccount);
}

export async function fetchAllProposalVote(
  rpc: Parameters<typeof fetchEncodedAccounts>[0],
  addresses: Array<Address>,
  config?: FetchAccountsConfig
): Promise<Account<ProposalVote>[]> {
  const maybeAccounts = await fetchAllMaybeProposalVote(rpc, addresses, config);
  assertAccountsExist(maybeAccounts);
  return maybeAccounts;
}

export async function fetchAllMaybeProposalVote(
  rpc: Parameters<typeof fetchEncodedAccounts>[0],
  addresses: Array<Address>,
  config?: FetchAccountsConfig
): Promise<MaybeAccount<ProposalVote>[]> {
  const maybeAccounts = await fetchEncodedAccounts(rpc, addresses, config);
  return maybeAccounts.map((maybeAccount) => decodeProposalVote(maybeAccount));
}

export function getProposalVoteSize(): number {
  return 80;
}
