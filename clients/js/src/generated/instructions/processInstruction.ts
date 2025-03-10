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
  getU32Decoder,
  getU32Encoder,
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
  type WritableAccount,
} from '@solana/web3.js';
import { PALADIN_GOVERNANCE_PROGRAM_ADDRESS } from '../programs';
import { getAccountMetaFactory, type ResolvedAccount } from '../shared';

export type ProcessInstructionInstruction<
  TProgram extends string = typeof PALADIN_GOVERNANCE_PROGRAM_ADDRESS,
  TAccountProposal extends string | IAccountMeta<string> = string,
  TAccountProposalTransaction extends string | IAccountMeta<string> = string,
  TRemainingAccounts extends readonly IAccountMeta<string>[] = [],
> = IInstruction<TProgram> &
  IInstructionWithData<Uint8Array> &
  IInstructionWithAccounts<
    [
      TAccountProposal extends string
        ? WritableAccount<TAccountProposal>
        : TAccountProposal,
      TAccountProposalTransaction extends string
        ? WritableAccount<TAccountProposalTransaction>
        : TAccountProposalTransaction,
      ...TRemainingAccounts,
    ]
  >;

export type ProcessInstructionInstructionData = {
  discriminator: number;
  instructionIndex: number;
};

export type ProcessInstructionInstructionDataArgs = {
  instructionIndex: number;
};

export function getProcessInstructionInstructionDataEncoder(): Encoder<ProcessInstructionInstructionDataArgs> {
  return transformEncoder(
    getStructEncoder([
      ['discriminator', getU8Encoder()],
      ['instructionIndex', getU32Encoder()],
    ]),
    (value) => ({ ...value, discriminator: 9 })
  );
}

export function getProcessInstructionInstructionDataDecoder(): Decoder<ProcessInstructionInstructionData> {
  return getStructDecoder([
    ['discriminator', getU8Decoder()],
    ['instructionIndex', getU32Decoder()],
  ]);
}

export function getProcessInstructionInstructionDataCodec(): Codec<
  ProcessInstructionInstructionDataArgs,
  ProcessInstructionInstructionData
> {
  return combineCodec(
    getProcessInstructionInstructionDataEncoder(),
    getProcessInstructionInstructionDataDecoder()
  );
}

export type ProcessInstructionInput<
  TAccountProposal extends string = string,
  TAccountProposalTransaction extends string = string,
> = {
  /** Proposal account */
  proposal: Address<TAccountProposal>;
  /** Proposal transaction account */
  proposalTransaction: Address<TAccountProposalTransaction>;
  instructionIndex: ProcessInstructionInstructionDataArgs['instructionIndex'];
};

export function getProcessInstructionInstruction<
  TAccountProposal extends string,
  TAccountProposalTransaction extends string,
>(
  input: ProcessInstructionInput<TAccountProposal, TAccountProposalTransaction>
): ProcessInstructionInstruction<
  typeof PALADIN_GOVERNANCE_PROGRAM_ADDRESS,
  TAccountProposal,
  TAccountProposalTransaction
> {
  // Program address.
  const programAddress = PALADIN_GOVERNANCE_PROGRAM_ADDRESS;

  // Original accounts.
  const originalAccounts = {
    proposal: { value: input.proposal ?? null, isWritable: true },
    proposalTransaction: {
      value: input.proposalTransaction ?? null,
      isWritable: true,
    },
  };
  const accounts = originalAccounts as Record<
    keyof typeof originalAccounts,
    ResolvedAccount
  >;

  // Original args.
  const args = { ...input };

  const getAccountMeta = getAccountMetaFactory(programAddress, 'programId');
  const instruction = {
    accounts: [
      getAccountMeta(accounts.proposal),
      getAccountMeta(accounts.proposalTransaction),
    ],
    programAddress,
    data: getProcessInstructionInstructionDataEncoder().encode(
      args as ProcessInstructionInstructionDataArgs
    ),
  } as ProcessInstructionInstruction<
    typeof PALADIN_GOVERNANCE_PROGRAM_ADDRESS,
    TAccountProposal,
    TAccountProposalTransaction
  >;

  return instruction;
}

export type ParsedProcessInstructionInstruction<
  TProgram extends string = typeof PALADIN_GOVERNANCE_PROGRAM_ADDRESS,
  TAccountMetas extends readonly IAccountMeta[] = readonly IAccountMeta[],
> = {
  programAddress: Address<TProgram>;
  accounts: {
    /** Proposal account */
    proposal: TAccountMetas[0];
    /** Proposal transaction account */
    proposalTransaction: TAccountMetas[1];
  };
  data: ProcessInstructionInstructionData;
};

export function parseProcessInstructionInstruction<
  TProgram extends string,
  TAccountMetas extends readonly IAccountMeta[],
>(
  instruction: IInstruction<TProgram> &
    IInstructionWithAccounts<TAccountMetas> &
    IInstructionWithData<Uint8Array>
): ParsedProcessInstructionInstruction<TProgram, TAccountMetas> {
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
      proposal: getNextAccount(),
      proposalTransaction: getNextAccount(),
    },
    data: getProcessInstructionInstructionDataDecoder().decode(
      instruction.data
    ),
  };
}
