/**
 * This code was AUTOGENERATED using the kinobi library.
 * Please DO NOT EDIT THIS FILE, instead use visitors
 * to add features, then rerun kinobi to update it.
 *
 * @see https://github.com/kinobi-so/kinobi
 */

import {
  containsBytes,
  getU8Encoder,
  type Address,
  type ReadonlyUint8Array,
} from '@solana/web3.js';
import {
  type ParsedBeginVotingInstruction,
  type ParsedCreateProposalInstruction,
  type ParsedDeleteProposalInstruction,
  type ParsedFinishVotingInstruction,
  type ParsedInitializeGovernanceInstruction,
  type ParsedProcessInstructionInstruction,
  type ParsedPushInstructionInstruction,
  type ParsedRemoveInstructionInstruction,
  type ParsedSwitchVoteInstruction,
  type ParsedUpdateGovernanceInstruction,
  type ParsedVoteInstruction,
} from '../instructions';

export const PALADIN_GOVERNANCE_PROGRAM_ADDRESS =
  'C1iuSykZ3SbTPmzZy66L57yQm6xnAtVdqEgYw2V39ptJ' as Address<'C1iuSykZ3SbTPmzZy66L57yQm6xnAtVdqEgYw2V39ptJ'>;

export enum PaladinGovernanceAccount {
  GovernanceConfig,
  Proposal,
  ProposalVote,
  Author,
}

export enum PaladinGovernanceInstruction {
  CreateProposal,
  PushInstruction,
  RemoveInstruction,
  DeleteProposal,
  BeginVoting,
  Vote,
  SwitchVote,
  FinishVoting,
  ProcessInstruction,
  InitializeGovernance,
  UpdateGovernance,
}

export function identifyPaladinGovernanceInstruction(
  instruction: { data: ReadonlyUint8Array } | ReadonlyUint8Array
): PaladinGovernanceInstruction {
  const data = 'data' in instruction ? instruction.data : instruction;
  if (containsBytes(data, getU8Encoder().encode(0), 0)) {
    return PaladinGovernanceInstruction.CreateProposal;
  }
  if (containsBytes(data, getU8Encoder().encode(1), 0)) {
    return PaladinGovernanceInstruction.PushInstruction;
  }
  if (containsBytes(data, getU8Encoder().encode(2), 0)) {
    return PaladinGovernanceInstruction.RemoveInstruction;
  }
  if (containsBytes(data, getU8Encoder().encode(3), 0)) {
    return PaladinGovernanceInstruction.DeleteProposal;
  }
  if (containsBytes(data, getU8Encoder().encode(4), 0)) {
    return PaladinGovernanceInstruction.BeginVoting;
  }
  if (containsBytes(data, getU8Encoder().encode(5), 0)) {
    return PaladinGovernanceInstruction.Vote;
  }
  if (containsBytes(data, getU8Encoder().encode(6), 0)) {
    return PaladinGovernanceInstruction.SwitchVote;
  }
  if (containsBytes(data, getU8Encoder().encode(7), 0)) {
    return PaladinGovernanceInstruction.FinishVoting;
  }
  if (containsBytes(data, getU8Encoder().encode(8), 0)) {
    return PaladinGovernanceInstruction.ProcessInstruction;
  }
  if (containsBytes(data, getU8Encoder().encode(9), 0)) {
    return PaladinGovernanceInstruction.InitializeGovernance;
  }
  if (containsBytes(data, getU8Encoder().encode(10), 0)) {
    return PaladinGovernanceInstruction.UpdateGovernance;
  }
  throw new Error(
    'The provided instruction could not be identified as a paladinGovernance instruction.'
  );
}

export type ParsedPaladinGovernanceInstruction<
  TProgram extends string = 'C1iuSykZ3SbTPmzZy66L57yQm6xnAtVdqEgYw2V39ptJ',
> =
  | ({
      instructionType: PaladinGovernanceInstruction.CreateProposal;
    } & ParsedCreateProposalInstruction<TProgram>)
  | ({
      instructionType: PaladinGovernanceInstruction.PushInstruction;
    } & ParsedPushInstructionInstruction<TProgram>)
  | ({
      instructionType: PaladinGovernanceInstruction.RemoveInstruction;
    } & ParsedRemoveInstructionInstruction<TProgram>)
  | ({
      instructionType: PaladinGovernanceInstruction.DeleteProposal;
    } & ParsedDeleteProposalInstruction<TProgram>)
  | ({
      instructionType: PaladinGovernanceInstruction.BeginVoting;
    } & ParsedBeginVotingInstruction<TProgram>)
  | ({
      instructionType: PaladinGovernanceInstruction.Vote;
    } & ParsedVoteInstruction<TProgram>)
  | ({
      instructionType: PaladinGovernanceInstruction.SwitchVote;
    } & ParsedSwitchVoteInstruction<TProgram>)
  | ({
      instructionType: PaladinGovernanceInstruction.FinishVoting;
    } & ParsedFinishVotingInstruction<TProgram>)
  | ({
      instructionType: PaladinGovernanceInstruction.ProcessInstruction;
    } & ParsedProcessInstructionInstruction<TProgram>)
  | ({
      instructionType: PaladinGovernanceInstruction.InitializeGovernance;
    } & ParsedInitializeGovernanceInstruction<TProgram>)
  | ({
      instructionType: PaladinGovernanceInstruction.UpdateGovernance;
    } & ParsedUpdateGovernanceInstruction<TProgram>);
