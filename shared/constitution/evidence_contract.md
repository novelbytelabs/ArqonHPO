# Arqon Zero Evidence Contract

Version: 0.1.0

## Authoritative Format

Protobuf or another declared canonical machine-readable artifact is authoritative for infrastructure records.

JSON and Markdown are human-facing derived views only unless explicitly declared canonical by the Constitution and hash-chain policy.

## Required Evidence Fields

Every evidence bundle must include:

- Constitution version and hash
- Role constitution version and hash
- PM spec hash
- Coder patch hash, if applicable
- Helper execution hash, if applicable
- Auditor replay hash, if applicable
- Base commit
- Final commit
- Diff manifest
- Command logs
- Gate report
- Environment manifest
- Artifact manifest
- Required conservative status labels

## Hash Rule

Every stage hash must include:

- Stage type
- Schema/version hash
- Previous stage hash
- Canonical artifact bytes
- Blob manifest hash, if external blobs exist

## JSON Rule

Every JSON derived view must include:

- source_artifact_type
- source_artifact_hash
- renderer_name
- renderer_hash
- rendered_at
- is_authoritative: false
