/**
 * This code was AUTOGENERATED using the kinobi library.
 * Please DO NOT EDIT THIS FILE, instead use visitors
 * to add features, then rerun kinobi to update it.
 *
 * @see https://github.com/kinobi-so/kinobi
 */

import {
  combineCodec,
  getStructDecoder,
  getStructEncoder,
  getU8Decoder,
  getU8Encoder,
  transformEncoder,
  type Address,
  type Codec,
  type Decoder,
  type Encoder,
  type IAccountMeta,
  type IAccountSignerMeta,
  type IInstruction,
  type IInstructionWithAccounts,
  type IInstructionWithData,
  type ReadonlyAccount,
  type ReadonlySignerAccount,
  type TransactionSigner,
  type WritableAccount,
} from '@solana/web3.js';
import { PALADIN_GOVERNANCE_PROGRAM_PROGRAM_ADDRESS } from '../programs';
import { getAccountMetaFactory, type ResolvedAccount } from '../shared';
import {
  getProposalVoteElectionDecoder,
  getProposalVoteElectionEncoder,
  type ProposalVoteElection,
  type ProposalVoteElectionArgs,
} from '../types';

export type VoteInstruction<
  TProgram extends string = typeof PALADIN_GOVERNANCE_PROGRAM_PROGRAM_ADDRESS,
  TAccountStakeAuthority extends string | IAccountMeta<string> = string,
  TAccountStake extends string | IAccountMeta<string> = string,
  TAccountStakeConfig extends string | IAccountMeta<string> = string,
  TAccountVote extends string | IAccountMeta<string> = string,
  TAccountProposal extends string | IAccountMeta<string> = string,
  TAccountSystemProgram extends
    | string
    | IAccountMeta<string> = '11111111111111111111111111111111',
  TRemainingAccounts extends readonly IAccountMeta<string>[] = [],
> = IInstruction<TProgram> &
  IInstructionWithData<Uint8Array> &
  IInstructionWithAccounts<
    [
      TAccountStakeAuthority extends string
        ? ReadonlySignerAccount<TAccountStakeAuthority> &
            IAccountSignerMeta<TAccountStakeAuthority>
        : TAccountStakeAuthority,
      TAccountStake extends string
        ? ReadonlyAccount<TAccountStake>
        : TAccountStake,
      TAccountStakeConfig extends string
        ? ReadonlyAccount<TAccountStakeConfig>
        : TAccountStakeConfig,
      TAccountVote extends string
        ? WritableAccount<TAccountVote>
        : TAccountVote,
      TAccountProposal extends string
        ? WritableAccount<TAccountProposal>
        : TAccountProposal,
      TAccountSystemProgram extends string
        ? ReadonlyAccount<TAccountSystemProgram>
        : TAccountSystemProgram,
      ...TRemainingAccounts,
    ]
  >;

export type VoteInstructionData = {
  discriminator: number;
  election: ProposalVoteElection;
};

export type VoteInstructionDataArgs = { election: ProposalVoteElectionArgs };

export function getVoteInstructionDataEncoder(): Encoder<VoteInstructionDataArgs> {
  return transformEncoder(
    getStructEncoder([
      ['discriminator', getU8Encoder()],
      ['election', getProposalVoteElectionEncoder()],
    ]),
    (value) => ({ ...value, discriminator: 5 })
  );
}

export function getVoteInstructionDataDecoder(): Decoder<VoteInstructionData> {
  return getStructDecoder([
    ['discriminator', getU8Decoder()],
    ['election', getProposalVoteElectionDecoder()],
  ]);
}

export function getVoteInstructionDataCodec(): Codec<
  VoteInstructionDataArgs,
  VoteInstructionData
> {
  return combineCodec(
    getVoteInstructionDataEncoder(),
    getVoteInstructionDataDecoder()
  );
}

export type VoteInput<
  TAccountStakeAuthority extends string = string,
  TAccountStake extends string = string,
  TAccountStakeConfig extends string = string,
  TAccountVote extends string = string,
  TAccountProposal extends string = string,
  TAccountSystemProgram extends string = string,
> = {
  /** Paladin stake authority account */
  stakeAuthority: TransactionSigner<TAccountStakeAuthority>;
  /** Paladin stake account */
  stake: Address<TAccountStake>;
  /** Paladin stake config account */
  stakeConfig: Address<TAccountStakeConfig>;
  /** Proposal vote account */
  vote: Address<TAccountVote>;
  /** Proposal account */
  proposal: Address<TAccountProposal>;
  /** System program */
  systemProgram?: Address<TAccountSystemProgram>;
  election: VoteInstructionDataArgs['election'];
};

export function getVoteInstruction<
  TAccountStakeAuthority extends string,
  TAccountStake extends string,
  TAccountStakeConfig extends string,
  TAccountVote extends string,
  TAccountProposal extends string,
  TAccountSystemProgram extends string,
>(
  input: VoteInput<
    TAccountStakeAuthority,
    TAccountStake,
    TAccountStakeConfig,
    TAccountVote,
    TAccountProposal,
    TAccountSystemProgram
  >
): VoteInstruction<
  typeof PALADIN_GOVERNANCE_PROGRAM_PROGRAM_ADDRESS,
  TAccountStakeAuthority,
  TAccountStake,
  TAccountStakeConfig,
  TAccountVote,
  TAccountProposal,
  TAccountSystemProgram
> {
  // Program address.
  const programAddress = PALADIN_GOVERNANCE_PROGRAM_PROGRAM_ADDRESS;

  // Original accounts.
  const originalAccounts = {
    stakeAuthority: { value: input.stakeAuthority ?? null, isWritable: false },
    stake: { value: input.stake ?? null, isWritable: false },
    stakeConfig: { value: input.stakeConfig ?? null, isWritable: false },
    vote: { value: input.vote ?? null, isWritable: true },
    proposal: { value: input.proposal ?? null, isWritable: true },
    systemProgram: { value: input.systemProgram ?? null, isWritable: false },
  };
  const accounts = originalAccounts as Record<
    keyof typeof originalAccounts,
    ResolvedAccount
  >;

  // Original args.
  const args = { ...input };

  // Resolve default values.
  if (!accounts.systemProgram.value) {
    accounts.systemProgram.value =
      '11111111111111111111111111111111' as Address<'11111111111111111111111111111111'>;
  }

  const getAccountMeta = getAccountMetaFactory(programAddress, 'programId');
  const instruction = {
    accounts: [
      getAccountMeta(accounts.stakeAuthority),
      getAccountMeta(accounts.stake),
      getAccountMeta(accounts.stakeConfig),
      getAccountMeta(accounts.vote),
      getAccountMeta(accounts.proposal),
      getAccountMeta(accounts.systemProgram),
    ],
    programAddress,
    data: getVoteInstructionDataEncoder().encode(
      args as VoteInstructionDataArgs
    ),
  } as VoteInstruction<
    typeof PALADIN_GOVERNANCE_PROGRAM_PROGRAM_ADDRESS,
    TAccountStakeAuthority,
    TAccountStake,
    TAccountStakeConfig,
    TAccountVote,
    TAccountProposal,
    TAccountSystemProgram
  >;

  return instruction;
}

export type ParsedVoteInstruction<
  TProgram extends string = typeof PALADIN_GOVERNANCE_PROGRAM_PROGRAM_ADDRESS,
  TAccountMetas extends readonly IAccountMeta[] = readonly IAccountMeta[],
> = {
  programAddress: Address<TProgram>;
  accounts: {
    /** Paladin stake authority account */
    stakeAuthority: TAccountMetas[0];
    /** Paladin stake account */
    stake: TAccountMetas[1];
    /** Paladin stake config account */
    stakeConfig: TAccountMetas[2];
    /** Proposal vote account */
    vote: TAccountMetas[3];
    /** Proposal account */
    proposal: TAccountMetas[4];
    /** System program */
    systemProgram: TAccountMetas[5];
  };
  data: VoteInstructionData;
};

export function parseVoteInstruction<
  TProgram extends string,
  TAccountMetas extends readonly IAccountMeta[],
>(
  instruction: IInstruction<TProgram> &
    IInstructionWithAccounts<TAccountMetas> &
    IInstructionWithData<Uint8Array>
): ParsedVoteInstruction<TProgram, TAccountMetas> {
  if (instruction.accounts.length < 6) {
    // TODO: Coded error.
    throw new Error('Not enough accounts');
  }
  let accountIndex = 0;
  const getNextAccount = () => {
    const accountMeta = instruction.accounts![accountIndex]!;
    accountIndex += 1;
    return accountMeta;
  };
  return {
    programAddress: instruction.programAddress,
    accounts: {
      stakeAuthority: getNextAccount(),
      stake: getNextAccount(),
      stakeConfig: getNextAccount(),
      vote: getNextAccount(),
      proposal: getNextAccount(),
      systemProgram: getNextAccount(),
    },
    data: getVoteInstructionDataDecoder().decode(instruction.data),
  };
}