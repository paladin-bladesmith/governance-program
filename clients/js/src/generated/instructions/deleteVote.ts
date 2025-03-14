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

export type DeleteVoteInstruction<
  TProgram extends string = typeof PALADIN_GOVERNANCE_PROGRAM_ADDRESS,
  TAccountProposal extends string | IAccountMeta<string> = string,
  TAccountVote extends string | IAccountMeta<string> = string,
  TAccountAuthority extends string | IAccountMeta<string> = string,
  TRemainingAccounts extends readonly IAccountMeta<string>[] = [],
> = IInstruction<TProgram> &
  IInstructionWithData<Uint8Array> &
  IInstructionWithAccounts<
    [
      TAccountProposal extends string
        ? ReadonlyAccount<TAccountProposal>
        : TAccountProposal,
      TAccountVote extends string
        ? WritableAccount<TAccountVote>
        : TAccountVote,
      TAccountAuthority extends string
        ? WritableAccount<TAccountAuthority>
        : TAccountAuthority,
      ...TRemainingAccounts,
    ]
  >;

export type DeleteVoteInstructionData = { discriminator: number };

export type DeleteVoteInstructionDataArgs = {};

export function getDeleteVoteInstructionDataEncoder(): Encoder<DeleteVoteInstructionDataArgs> {
  return transformEncoder(
    getStructEncoder([['discriminator', getU8Encoder()]]),
    (value) => ({ ...value, discriminator: 8 })
  );
}

export function getDeleteVoteInstructionDataDecoder(): Decoder<DeleteVoteInstructionData> {
  return getStructDecoder([['discriminator', getU8Decoder()]]);
}

export function getDeleteVoteInstructionDataCodec(): Codec<
  DeleteVoteInstructionDataArgs,
  DeleteVoteInstructionData
> {
  return combineCodec(
    getDeleteVoteInstructionDataEncoder(),
    getDeleteVoteInstructionDataDecoder()
  );
}

export type DeleteVoteInput<
  TAccountProposal extends string = string,
  TAccountVote extends string = string,
  TAccountAuthority extends string = string,
> = {
  proposal: Address<TAccountProposal>;
  vote: Address<TAccountVote>;
  authority: Address<TAccountAuthority>;
};

export function getDeleteVoteInstruction<
  TAccountProposal extends string,
  TAccountVote extends string,
  TAccountAuthority extends string,
>(
  input: DeleteVoteInput<TAccountProposal, TAccountVote, TAccountAuthority>
): DeleteVoteInstruction<
  typeof PALADIN_GOVERNANCE_PROGRAM_ADDRESS,
  TAccountProposal,
  TAccountVote,
  TAccountAuthority
> {
  // Program address.
  const programAddress = PALADIN_GOVERNANCE_PROGRAM_ADDRESS;

  // Original accounts.
  const originalAccounts = {
    proposal: { value: input.proposal ?? null, isWritable: false },
    vote: { value: input.vote ?? null, isWritable: true },
    authority: { value: input.authority ?? null, isWritable: true },
  };
  const accounts = originalAccounts as Record<
    keyof typeof originalAccounts,
    ResolvedAccount
  >;

  const getAccountMeta = getAccountMetaFactory(programAddress, 'programId');
  const instruction = {
    accounts: [
      getAccountMeta(accounts.proposal),
      getAccountMeta(accounts.vote),
      getAccountMeta(accounts.authority),
    ],
    programAddress,
    data: getDeleteVoteInstructionDataEncoder().encode({}),
  } as DeleteVoteInstruction<
    typeof PALADIN_GOVERNANCE_PROGRAM_ADDRESS,
    TAccountProposal,
    TAccountVote,
    TAccountAuthority
  >;

  return instruction;
}

export type ParsedDeleteVoteInstruction<
  TProgram extends string = typeof PALADIN_GOVERNANCE_PROGRAM_ADDRESS,
  TAccountMetas extends readonly IAccountMeta[] = readonly IAccountMeta[],
> = {
  programAddress: Address<TProgram>;
  accounts: {
    proposal: TAccountMetas[0];
    vote: TAccountMetas[1];
    authority: TAccountMetas[2];
  };
  data: DeleteVoteInstructionData;
};

export function parseDeleteVoteInstruction<
  TProgram extends string,
  TAccountMetas extends readonly IAccountMeta[],
>(
  instruction: IInstruction<TProgram> &
    IInstructionWithAccounts<TAccountMetas> &
    IInstructionWithData<Uint8Array>
): ParsedDeleteVoteInstruction<TProgram, TAccountMetas> {
  if (instruction.accounts.length < 3) {
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
      proposal: getNextAccount(),
      vote: getNextAccount(),
      authority: getNextAccount(),
    },
    data: getDeleteVoteInstructionDataDecoder().decode(instruction.data),
  };
}
