#!/usr/bin/env zx
import "zx/globals";
import * as k from "kinobi";
import { rootNodeFromAnchor } from "@kinobi-so/nodes-from-anchor";
import { renderVisitor as renderJavaScriptVisitor } from "@kinobi-so/renderers-js";
import { renderVisitor as renderRustVisitor } from "@kinobi-so/renderers-rust";
import { getAllProgramIdls, getToolchainArgument } from "./utils.mjs";

// Instanciate Kinobi.
const [idl, ...additionalIdls] = getAllProgramIdls().map(idl => rootNodeFromAnchor(require(idl)))
const kinobi = k.createFromRoot(idl, additionalIdls);

// Add missing types from the IDL.
kinobi.update(
  k.bottomUpTransformerVisitor([
    {
      select: "[programNode]paladinGovernanceProgram",
      transform: (node) => {
        k.assertIsNode(node, "programNode");
        return {
          ...node,
          definedTypes: [
            ...node.definedTypes,
            // GovernanceConfig redefined for use in Proposal account state.
            k.definedTypeNode({
              name: "config",
              type: k.structTypeNode([
                k.structFieldTypeNode({
                  name: "cooldownPeriodSeconds",
                  type: k.numberTypeNode("u64"),
                }),
                k.structFieldTypeNode({
                  name: "proposalAcceptanceThreshold",
                  type: k.numberTypeNode("u32"),
                }),
                k.structFieldTypeNode({
                  name: "proposalRejectionThreshold",
                  type: k.numberTypeNode("u32"),
                }),
                k.structFieldTypeNode({
                  name: "signerBumpSeed",
                  type: k.numberTypeNode("u8"),
                }),
                k.structFieldTypeNode({
                  name: "_padding",
                  type: k.arrayTypeNode(
                    k.numberTypeNode("u8"),
                    k.fixedCountNode(7),
                  ),
                }),
                k.structFieldTypeNode({
                  name: "stakeConfigAddress",
                  type: k.publicKeyTypeNode(),
                }),
                k.structFieldTypeNode({
                  name: "votingPeriodSeconds",
                  type: k.numberTypeNode("u64"),
                }),
              ]),
            }),
          ],
        }
      }
    },
    {
      // GovernanceConfig -> Config
      select: "[structFieldTypeNode]governanceConfig",
      transform: (node) => {
        k.assertIsNode(node, "structFieldTypeNode");
        return {
          ...node,
          type: k.definedTypeLinkNode("config"),
        };
      },
    },
    {
      // Option<NonZeroU64> -> NullableU64
      select: "[structFieldTypeNode]cooldownTimestamp",
      transform: (node) => {
        k.assertIsNode(node, "structFieldTypeNode");
        return {
          ...node,
          type: k.definedTypeLinkNode("nullableU64", "hooked"),
        };
      },
    },
    {
      // Option<NonZeroU64> -> NullableU64
      select: "[structFieldTypeNode]votingStartTimestamp",
      transform: (node) => {
        k.assertIsNode(node, "structFieldTypeNode");
        return {
          ...node,
          type: k.definedTypeLinkNode("nullableU64", "hooked"),
        };
      },
    },
    {
      // UnixTimestamp -> i64
      select: "[structFieldTypeNode]creationTimestamp",
      transform: (node) => {
        k.assertIsNode(node, "structFieldTypeNode");
        return {
          ...node,
          type: k.numberTypeNode("i64"),
        };
      },
    },
  ])
);

// Render JavaScript.
const jsClient = path.join(__dirname, "..", "clients", "js");
kinobi.accept(
  renderJavaScriptVisitor(path.join(jsClient, "src", "generated"), { 
    prettier: require(path.join(jsClient, ".prettierrc.json"))
  })
);

// Render Rust.
const rustClient = path.join(__dirname, "..", "clients", "rust");
kinobi.accept(
  renderRustVisitor(path.join(rustClient, "src", "generated"), {
    formatCode: true,
    crateFolder: rustClient,
    toolchain: getToolchainArgument('format')
  })
);
