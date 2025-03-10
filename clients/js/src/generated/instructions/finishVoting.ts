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
  type IInstruction,
  type IInstructionWithAccounts,
  type IInstructionWithData,
  type ReadonlyAccount,
  type WritableAccount,
} from '@solana/web3.js';
import { PALADIN_GOVERNANCE_PROGRAM_ADDRESS } from '../programs';
import { getAccountMetaFactory, type ResolvedAccount } from '../shared';

export type FinishVotingInstruction<
  TProgram extends string = typeof PALADIN_GOVERNANCE_PROGRAM_ADDRESS,
  TAccountStakeConfig extends string | IAccountMeta<string> = string,
  TAccountProposal extends string | IAccountMeta<string> = string,
  TRemainingAccounts extends readonly IAccountMeta<string>[] = [],
> = IInstruction<TProgram> &
  IInstructionWithData<Uint8Array> &
  IInstructionWithAccounts<
    [
      TAccountStakeConfig extends string
        ? ReadonlyAccount<TAccountStakeConfig>
        : TAccountStakeConfig,
      TAccountProposal extends string
        ? WritableAccount<TAccountProposal>
        : TAccountProposal,
      ...TRemainingAccounts,
    ]
  >;

export type FinishVotingInstructionData = { discriminator: number };

export type FinishVotingInstructionDataArgs = {};

export function getFinishVotingInstructionDataEncoder(): Encoder<FinishVotingInstructionDataArgs> {
  return transformEncoder(
    getStructEncoder([['discriminator', getU8Encoder()]]),
    (value) => ({ ...value, discriminator: 7 })
  );
}

export function getFinishVotingInstructionDataDecoder(): Decoder<FinishVotingInstructionData> {
  return getStructDecoder([['discriminator', getU8Decoder()]]);
}

export function getFinishVotingInstructionDataCodec(): Codec<
  FinishVotingInstructionDataArgs,
  FinishVotingInstructionData
> {
  return combineCodec(
    getFinishVotingInstructionDataEncoder(),
    getFinishVotingInstructionDataDecoder()
  );
}

export type FinishVotingInput<
  TAccountStakeConfig extends string = string,
  TAccountProposal extends string = string,
> = {
  /** Stake config account */
  stakeConfig: Address<TAccountStakeConfig>;
  /** Proposal account */
  proposal: Address<TAccountProposal>;
};

export function getFinishVotingInstruction<
  TAccountStakeConfig extends string,
  TAccountProposal extends string,
>(
  input: FinishVotingInput<TAccountStakeConfig, TAccountProposal>
): FinishVotingInstruction<
  typeof PALADIN_GOVERNANCE_PROGRAM_ADDRESS,
  TAccountStakeConfig,
  TAccountProposal
> {
  // Program address.
  const programAddress = PALADIN_GOVERNANCE_PROGRAM_ADDRESS;

  // Original accounts.
  const originalAccounts = {
    stakeConfig: { value: input.stakeConfig ?? null, isWritable: false },
    proposal: { value: input.proposal ?? null, isWritable: true },
  };
  const accounts = originalAccounts as Record<
    keyof typeof originalAccounts,
    ResolvedAccount
  >;

  const getAccountMeta = getAccountMetaFactory(programAddress, 'programId');
  const instruction = {
    accounts: [
      getAccountMeta(accounts.stakeConfig),
      getAccountMeta(accounts.proposal),
    ],
    programAddress,
    data: getFinishVotingInstructionDataEncoder().encode({}),
  } as FinishVotingInstruction<
    typeof PALADIN_GOVERNANCE_PROGRAM_ADDRESS,
    TAccountStakeConfig,
    TAccountProposal
  >;

  return instruction;
}

export type ParsedFinishVotingInstruction<
  TProgram extends string = typeof PALADIN_GOVERNANCE_PROGRAM_ADDRESS,
  TAccountMetas extends readonly IAccountMeta[] = readonly IAccountMeta[],
> = {
  programAddress: Address<TProgram>;
  accounts: {
    /** Stake config account */
    stakeConfig: TAccountMetas[0];
    /** Proposal account */
    proposal: TAccountMetas[1];
  };
  data: FinishVotingInstructionData;
};

export function parseFinishVotingInstruction<
  TProgram extends string,
  TAccountMetas extends readonly IAccountMeta[],
>(
  instruction: IInstruction<TProgram> &
    IInstructionWithAccounts<TAccountMetas> &
    IInstructionWithData<Uint8Array>
): ParsedFinishVotingInstruction<TProgram, TAccountMetas> {
  if (instruction.accounts.length < 2) {
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
      stakeConfig: getNextAccount(),
      proposal: getNextAccount(),
    },
    data: getFinishVotingInstructionDataDecoder().decode(instruction.data),
  };
}
