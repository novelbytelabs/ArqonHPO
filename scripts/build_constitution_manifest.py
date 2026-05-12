#!/usr/bin/env python3
"""Build Arqon Zero static governance JSON files for GitHub Pages.

Run from repo root:

    python3 scripts/build_constitution_manifest.py

Expected source files:

    shared/constitution/shared_gpt_constitution.md
    shared/constitution/auditor_gpt_constitution.md
    shared/constitution/status_language_policy.md
    shared/constitution/evidence_contract.md
    shared/constitution/sealed_test_boundary.md
    shared/constitution/audit_replay_policy.md
    shared/constitution/constitution_amendment_protocol.md

Outputs:

    shared/constitution/auditor_gpt_context.json
    shared/constitution/constitution_manifest.json
    shared/constitution/status_language_policy.json
    shared/constitution/amendment_protocol.json
"""

from __future__ import annotations

import hashlib
import json
from datetime import datetime, timezone
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CONSTITUTION_DIR = ROOT / "shared" / "constitution"
URL_BASE = "https://novelbytelabs.github.io/ArqonHPO"


def sha256_text(text: str) -> str:
    return "sha256:" + hashlib.sha256(text.encode("utf-8")).hexdigest()


def read_doc(filename: str, name: str, version: str = "0.1.0") -> dict:
    path = CONSTITUTION_DIR / filename
    content = path.read_text(encoding="utf-8")
    return {
        "name": name,
        "version": version,
        "hash": sha256_text(content),
        "path": f"shared/constitution/{filename}",
        "url": f"{URL_BASE}/shared/constitution/{filename}",
        "content": content,
    }


def write_json(filename: str, payload: dict) -> str:
    text = json.dumps(payload, indent=2, sort_keys=True, ensure_ascii=False) + "\n"
    path = CONSTITUTION_DIR / filename
    path.write_text(text, encoding="utf-8")
    return sha256_text(text)


def main() -> None:
    shared = read_doc("shared_gpt_constitution.md", "ARQON_ZERO_SHARED_GPT_CONSTITUTION")
    auditor = read_doc("auditor_gpt_constitution.md", "AUDITOR_AI_GPT_CONSTITUTION")

    policy_files = [
        ("status_language_policy.md", "STATUS_LANGUAGE_POLICY"),
        ("evidence_contract.md", "EVIDENCE_CONTRACT"),
        ("sealed_test_boundary.md", "SEALED_TEST_BOUNDARY"),
        ("audit_replay_policy.md", "AUDIT_REPLAY_POLICY"),
        ("constitution_amendment_protocol.md", "CONSTITUTION_AMENDMENT_PROTOCOL"),
    ]
    policies = [read_doc(filename, name) for filename, name in policy_files]

    status_language = {
        "required_labels": [
            "REQUIRES_HUMAN_REVIEW",
            "development diagnostic only",
            "NOT SEALED-TEST CERTIFIED",
            "not promotable",
        ],
        "stale_fallback_label": "REQUIRES_STALENESS_REVIEW",
        "removal_is_hard_fail": True,
    }

    amendment_protocol = {
        "required_flow": [
            "PM amendment proposal",
            "Coder patch draft",
            "Helper/Codex execution and governance checks",
            "Auditor weakening/conflict review",
            "Human ratification",
            "Version bump and manifest update",
        ],
        "hard_constraints": [
            "No removal of human final promotion authority",
            "No AI self-certification",
            "No silent weakening of sealed-test boundary",
            "No retroactive certification of failed/contaminated/diagnostic evidence",
            "No removal of conservative status language",
            "No bypass of audit replay requirements",
        ],
        "retroactive_certification_forbidden": True,
    }

    manifest_base = {
        "shared_constitution_version": shared["version"],
        "shared_constitution_hash": shared["hash"],
        "role_constitution_hashes": {
            "AUDITOR_AI": auditor["hash"],
        },
        "policy_hashes": {policy["name"]: policy["hash"] for policy in policies},
        "effective_url_base": URL_BASE,
        "generated_at": datetime.now(timezone.utc).isoformat(),
    }

    context = {
        "role": "AUDITOR_AI",
        "shared_constitution": shared,
        "role_constitution": auditor,
        "policies": policies,
        "manifest": dict(manifest_base),
        "status_language": status_language,
        "authority_order": [
            "Human final promotion authority",
            "Arqon Zero Shared GPT Constitution",
            "Auditor AI GPT Constitution",
            "Governance policies",
            "PM spec / audit request",
            "User request",
            "Auditor convenience or preference",
        ],
        "fallback_policy": {
            "live_load_required_for_governance_sensitive_tasks": True,
            "stale_fallback_allowed": True,
            "stale_fallback_status": "REQUIRES_STALENESS_REVIEW",
        },
    }

    status_hash = write_json("status_language_policy.json", status_language)
    amendment_hash = write_json("amendment_protocol.json", amendment_protocol)

    manifest = dict(manifest_base)
    manifest["derived_json_hashes"] = {
        "status_language_policy.json": status_hash,
        "amendment_protocol.json": amendment_hash,
    }

    context_text = json.dumps(context, indent=2, sort_keys=True, ensure_ascii=False) + "\n"
    context_hash = sha256_text(context_text)
    context["manifest"]["context_hash"] = context_hash

    context_hash = write_json("auditor_gpt_context.json", context)
    manifest["derived_json_hashes"]["auditor_gpt_context.json"] = context_hash

    manifest_text_without_hash = json.dumps(manifest, indent=2, sort_keys=True, ensure_ascii=False) + "\n"
    manifest["manifest_hash"] = sha256_text(manifest_text_without_hash)
    write_json("constitution_manifest.json", manifest)

    print("Wrote governance context files:")
    print(" - shared/constitution/auditor_gpt_context.json")
    print(" - shared/constitution/constitution_manifest.json")
    print(" - shared/constitution/status_language_policy.json")
    print(" - shared/constitution/amendment_protocol.json")


if __name__ == "__main__":
    main()
