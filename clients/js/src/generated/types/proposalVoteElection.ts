/**
 * This code was AUTOGENERATED using the kinobi library.
 * Please DO NOT EDIT THIS FILE, instead use visitors
 * to add features, then rerun kinobi to update it.
 *
 * @see https://github.com/kinobi-so/kinobi
 */

import {
  combineCodec,
  getEnumDecoder,
  getEnumEncoder,
  type Codec,
  type Decoder,
  type Encoder,
} from '@solana/web3.js';

export enum ProposalVoteElection {
  For,
  Against,
}

export type ProposalVoteElectionArgs = ProposalVoteElection;

export function getProposalVoteElectionEncoder(): Encoder<ProposalVoteElectionArgs> {
  return getEnumEncoder(ProposalVoteElection);
}

export function getProposalVoteElectionDecoder(): Decoder<ProposalVoteElection> {
  return getEnumDecoder(ProposalVoteElection);
}

export function getProposalVoteElectionCodec(): Codec<
  ProposalVoteElectionArgs,
  ProposalVoteElection
> {
  return combineCodec(
    getProposalVoteElectionEncoder(),
    getProposalVoteElectionDecoder()
  );
}
