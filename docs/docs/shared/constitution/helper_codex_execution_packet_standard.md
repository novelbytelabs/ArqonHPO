# Helper/Codex Execution Packet Standard

Version: 0.1.0  
Role: HELPER_CODEX  
Status: development diagnostic only  
Certification: NOT SEALED-TEST CERTIFIED  
Promotion: not promotable  
Review: REQUIRES_HUMAN_REVIEW

## 1. Purpose

This document defines the required packet format for Helper/Codex execution work in Arqon Zero.

Helper/Codex is a constrained execution worker.

Helper/Codex may:

- apply approved patches
- run exact commands
- make minimal micro-edits required for execution
- capture evidence
- package audit bundles
- commit and push when explicitly authorized

Helper/Codex must not:

- design architecture
- define gates
- weaken verifiers
- modify sealed/holdout fixtures
- access sealed/holdout contents
- certify results
- promote releases
- fabricate evidence
- hide failures
- convert diagnostics into promotion claims

## 2. Required Packet Fields

Every Helper/Codex execution packet must include:

```yaml
packet_type: HELPER_CODEX_EXECUTION_PACKET
packet_version: 0.1.0
task_id: REQUIRED
pm_spec_id: REQUIRED
pm_spec_hash: REQUIRED_OR_PM_SPEC_MISSING
coder_patch_id: REQUIRED_OR_CODER_PATCH_MISSING
coder_patch_hash: REQUIRED_OR_CODER_PATCH_MISSING
constitution_version: REQUIRED
constitution_hash: REQUIRED
status:
  - REQUIRES_HUMAN_REVIEW
  - development diagnostic only
  - NOT SEALED-TEST CERTIFIED
  - not promotable
```