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
  getU64Decoder,
  getU64Encoder,
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
} from '@solana/web3.js';
import { PALADIN_GOVERNANCE_PROGRAM_ADDRESS } from '../programs';
import { getAccountMetaFactory, type ResolvedAccount } from '../shared';

export type UpdateGovernanceInstruction<
  TProgram extends string = typeof PALADIN_GOVERNANCE_PROGRAM_ADDRESS,
  TAccountTreasury extends string | IAccountMeta<string> = string,
  TAccountGovernanceConfig extends string | IAccountMeta<string> = string,
  TRemainingAccounts extends readonly IAccountMeta<string>[] = [],
> = IInstruction<TProgram> &
  IInstructionWithData<Uint8Array> &
  IInstructionWithAccounts<
    [
      TAccountTreasury extends string
        ? ReadonlySignerAccount<TAccountTreasury> &
            IAccountSignerMeta<TAccountTreasury>
        : TAccountTreasury,
      TAccountGovernanceConfig extends string
        ? ReadonlyAccount<TAccountGovernanceConfig>
        : TAccountGovernanceConfig,
      ...TRemainingAccounts,
    ]
  >;

export type UpdateGovernanceInstructionData = {
  discriminator: number;
  governanceId: bigint;
  cooldownPeriodSeconds: bigint;
  proposalMinimumQuorum: number;
  proposalPassThreshold: number;
  votingPeriodSeconds: bigint;
  stakePerProposal: bigint;
};

export type UpdateGovernanceInstructionDataArgs = {
  governanceId: number | bigint;
  cooldownPeriodSeconds: number | bigint;
  proposalMinimumQuorum: number;
  proposalPassThreshold: number;
  votingPeriodSeconds: number | bigint;
  stakePerProposal: number | bigint;
};

export function getUpdateGovernanceInstructionDataEncoder(): Encoder<UpdateGovernanceInstructionDataArgs> {
  return transformEncoder(
    getStructEncoder([
      ['discriminator', getU8Encoder()],
      ['governanceId', getU64Encoder()],
      ['cooldownPeriodSeconds', getU64Encoder()],
      ['proposalMinimumQuorum', getU32Encoder()],
      ['proposalPassThreshold', getU32Encoder()],
      ['votingPeriodSeconds', getU64Encoder()],
      ['stakePerProposal', getU64Encoder()],
    ]),
    (value) => ({ ...value, discriminator: 11 })
  );
}

export function getUpdateGovernanceInstructionDataDecoder(): Decoder<UpdateGovernanceInstructionData> {
  return getStructDecoder([
    ['discriminator', getU8Decoder()],
    ['governanceId', getU64Decoder()],
    ['cooldownPeriodSeconds', getU64Decoder()],
    ['proposalMinimumQuorum', getU32Decoder()],
    ['proposalPassThreshold', getU32Decoder()],
    ['votingPeriodSeconds', getU64Decoder()],
    ['stakePerProposal', getU64Decoder()],
  ]);
}

export function getUpdateGovernanceInstructionDataCodec(): Codec<
  UpdateGovernanceInstructionDataArgs,
  UpdateGovernanceInstructionData
> {
  return combineCodec(
    getUpdateGovernanceInstructionDataEncoder(),
    getUpdateGovernanceInstructionDataDecoder()
  );
}

export type UpdateGovernanceInput<
  TAccountTreasury extends string = string,
  TAccountGovernanceConfig extends string = string,
> = {
  /** Treasury account */
  treasury: TransactionSigner<TAccountTreasury>;
  /** Governance config account */
  governanceConfig: Address<TAccountGovernanceConfig>;
  governanceId: UpdateGovernanceInstructionDataArgs['governanceId'];
  cooldownPeriodSeconds: UpdateGovernanceInstructionDataArgs['cooldownPeriodSeconds'];
  proposalMinimumQuorum: UpdateGovernanceInstructionDataArgs['proposalMinimumQuorum'];
  proposalPassThreshold: UpdateGovernanceInstructionDataArgs['proposalPassThreshold'];
  votingPeriodSeconds: UpdateGovernanceInstructionDataArgs['votingPeriodSeconds'];
  stakePerProposal: UpdateGovernanceInstructionDataArgs['stakePerProposal'];
};

export function getUpdateGovernanceInstruction<
  TAccountTreasury extends string,
  TAccountGovernanceConfig extends string,
>(
  input: UpdateGovernanceInput<TAccountTreasury, TAccountGovernanceConfig>
): UpdateGovernanceInstruction<
  typeof PALADIN_GOVERNANCE_PROGRAM_ADDRESS,
  TAccountTreasury,
  TAccountGovernanceConfig
> {
  // Program address.
  const programAddress = PALADIN_GOVERNANCE_PROGRAM_ADDRESS;

  // Original accounts.
  const originalAccounts = {
    treasury: { value: input.treasury ?? null, isWritable: false },
    governanceConfig: {
      value: input.governanceConfig ?? null,
      isWritable: false,
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
      getAccountMeta(accounts.treasury),
      getAccountMeta(accounts.governanceConfig),
    ],
    programAddress,
    data: getUpdateGovernanceInstructionDataEncoder().encode(
      args as UpdateGovernanceInstructionDataArgs
    ),
  } as UpdateGovernanceInstruction<
    typeof PALADIN_GOVERNANCE_PROGRAM_ADDRESS,
    TAccountTreasury,
    TAccountGovernanceConfig
  >;

  return instruction;
}

export type ParsedUpdateGovernanceInstruction<
  TProgram extends string = typeof PALADIN_GOVERNANCE_PROGRAM_ADDRESS,
  TAccountMetas extends readonly IAccountMeta[] = readonly IAccountMeta[],
> = {
  programAddress: Address<TProgram>;
  accounts: {
    /** Treasury account */
    treasury: TAccountMetas[0];
    /** Governance config account */
    governanceConfig: TAccountMetas[1];
  };
  data: UpdateGovernanceInstructionData;
};

export function parseUpdateGovernanceInstruction<
  TProgram extends string,
  TAccountMetas extends readonly IAccountMeta[],
>(
  instruction: IInstruction<TProgram> &
    IInstructionWithAccounts<TAccountMetas> &
    IInstructionWithData<Uint8Array>
): ParsedUpdateGovernanceInstruction<TProgram, TAccountMetas> {
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
      treasury: getNextAccount(),
      governanceConfig: getNextAccount(),
    },
    data: getUpdateGovernanceInstructionDataDecoder().decode(instruction.data),
  };
}
