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
  type TransactionSigner,
  type WritableAccount,
  type WritableSignerAccount,
} from '@solana/web3.js';
import { PALADIN_GOVERNANCE_PROGRAM_ADDRESS } from '../programs';
import { getAccountMetaFactory, type ResolvedAccount } from '../shared';

export type DeleteProposalInstruction<
  TProgram extends string = typeof PALADIN_GOVERNANCE_PROGRAM_ADDRESS,
  TAccountStakeAuthority extends string | IAccountMeta<string> = string,
  TAccountAuthor extends string | IAccountMeta<string> = string,
  TAccountProposal extends string | IAccountMeta<string> = string,
  TAccountProposalTransaction extends string | IAccountMeta<string> = string,
  TRemainingAccounts extends readonly IAccountMeta<string>[] = [],
> = IInstruction<TProgram> &
  IInstructionWithData<Uint8Array> &
  IInstructionWithAccounts<
    [
      TAccountStakeAuthority extends string
        ? WritableSignerAccount<TAccountStakeAuthority> &
            IAccountSignerMeta<TAccountStakeAuthority>
        : TAccountStakeAuthority,
      TAccountAuthor extends string
        ? WritableAccount<TAccountAuthor>
        : TAccountAuthor,
      TAccountProposal extends string
        ? WritableAccount<TAccountProposal>
        : TAccountProposal,
      TAccountProposalTransaction extends string
        ? WritableAccount<TAccountProposalTransaction>
        : TAccountProposalTransaction,
      ...TRemainingAccounts,
    ]
  >;

export type DeleteProposalInstructionData = { discriminator: number };

export type DeleteProposalInstructionDataArgs = {};

export function getDeleteProposalInstructionDataEncoder(): Encoder<DeleteProposalInstructionDataArgs> {
  return transformEncoder(
    getStructEncoder([['discriminator', getU8Encoder()]]),
    (value) => ({ ...value, discriminator: 4 })
  );
}

export function getDeleteProposalInstructionDataDecoder(): Decoder<DeleteProposalInstructionData> {
  return getStructDecoder([['discriminator', getU8Decoder()]]);
}

export function getDeleteProposalInstructionDataCodec(): Codec<
  DeleteProposalInstructionDataArgs,
  DeleteProposalInstructionData
> {
  return combineCodec(
    getDeleteProposalInstructionDataEncoder(),
    getDeleteProposalInstructionDataDecoder()
  );
}

export type DeleteProposalInput<
  TAccountStakeAuthority extends string = string,
  TAccountAuthor extends string = string,
  TAccountProposal extends string = string,
  TAccountProposalTransaction extends string = string,
> = {
  /** Paladin stake authority account */
  stakeAuthority: TransactionSigner<TAccountStakeAuthority>;
  /** Stake authority author account */
  author: Address<TAccountAuthor>;
  /** Proposal account */
  proposal: Address<TAccountProposal>;
  /** Proposal transaction account */
  proposalTransaction: Address<TAccountProposalTransaction>;
};

export function getDeleteProposalInstruction<
  TAccountStakeAuthority extends string,
  TAccountAuthor extends string,
  TAccountProposal extends string,
  TAccountProposalTransaction extends string,
>(
  input: DeleteProposalInput<
    TAccountStakeAuthority,
    TAccountAuthor,
    TAccountProposal,
    TAccountProposalTransaction
  >
): DeleteProposalInstruction<
  typeof PALADIN_GOVERNANCE_PROGRAM_ADDRESS,
  TAccountStakeAuthority,
  TAccountAuthor,
  TAccountProposal,
  TAccountProposalTransaction
> {
  // Program address.
  const programAddress = PALADIN_GOVERNANCE_PROGRAM_ADDRESS;

  // Original accounts.
  const originalAccounts = {
    stakeAuthority: { value: input.stakeAuthority ?? null, isWritable: true },
    author: { value: input.author ?? null, isWritable: true },
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

  const getAccountMeta = getAccountMetaFactory(programAddress, 'programId');
  const instruction = {
    accounts: [
      getAccountMeta(accounts.stakeAuthority),
      getAccountMeta(accounts.author),
      getAccountMeta(accounts.proposal),
      getAccountMeta(accounts.proposalTransaction),
    ],
    programAddress,
    data: getDeleteProposalInstructionDataEncoder().encode({}),
  } as DeleteProposalInstruction<
    typeof PALADIN_GOVERNANCE_PROGRAM_ADDRESS,
    TAccountStakeAuthority,
    TAccountAuthor,
    TAccountProposal,
    TAccountProposalTransaction
  >;

  return instruction;
}

export type ParsedDeleteProposalInstruction<
  TProgram extends string = typeof PALADIN_GOVERNANCE_PROGRAM_ADDRESS,
  TAccountMetas extends readonly IAccountMeta[] = readonly IAccountMeta[],
> = {
  programAddress: Address<TProgram>;
  accounts: {
    /** Paladin stake authority account */
    stakeAuthority: TAccountMetas[0];
    /** Stake authority author account */
    author: TAccountMetas[1];
    /** Proposal account */
    proposal: TAccountMetas[2];
    /** Proposal transaction account */
    proposalTransaction: TAccountMetas[3];
  };
  data: DeleteProposalInstructionData;
};

export function parseDeleteProposalInstruction<
  TProgram extends string,
  TAccountMetas extends readonly IAccountMeta[],
>(
  instruction: IInstruction<TProgram> &
    IInstructionWithAccounts<TAccountMetas> &
    IInstructionWithData<Uint8Array>
): ParsedDeleteProposalInstruction<TProgram, TAccountMetas> {
  if (instruction.accounts.length < 4) {
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
      author: getNextAccount(),
      proposal: getNextAccount(),
      proposalTransaction: getNextAccount(),
    },
    data: getDeleteProposalInstructionDataDecoder().decode(instruction.data),
  };
}
