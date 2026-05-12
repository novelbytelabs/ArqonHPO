#!/usr/bin/env python3
"""Build PM AI GPT governance context JSON for GitHub Pages.

Run from repo root:

    python3 scripts/build_pm_gpt_context.py

Inputs:
    shared/constitution/shared_gpt_constitution.md
    shared/constitution/pm_gpt_constitution.md
    shared/constitution/status_language_policy.md
    shared/constitution/evidence_contract.md
    shared/constitution/sealed_test_boundary.md
    shared/constitution/constitution_amendment_protocol.md
    shared/constitution/helper_codex_execution_charter.md

Output:
    shared/constitution/pm_gpt_context.json
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
    shared = read_doc(
        "shared_gpt_constitution.md",
        "ARQON_ZERO_SHARED_GPT_CONSTITUTION",
    )
    pm = read_doc(
        "pm_gpt_constitution.md",
        "PM_AI_GPT_CONSTITUTION",
    )

    policy_files = [
        ("status_language_policy.md", "STATUS_LANGUAGE_POLICY"),
        ("evidence_contract.md", "EVIDENCE_CONTRACT"),
        ("sealed_test_boundary.md", "SEALED_TEST_BOUNDARY"),
        ("constitution_amendment_protocol.md", "CONSTITUTION_AMENDMENT_PROTOCOL"),
        ("helper_codex_execution_charter.md", "HELPER_CODEX_EXECUTION_CHARTER"),
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

    manifest = {
        "shared_constitution_version": shared["version"],
        "shared_constitution_hash": shared["hash"],
        "role_constitution_hashes": {
            "PM_AI": pm["hash"],
        },
        "policy_hashes": {
            policy["name"]: policy["hash"] for policy in policies
        },
        "effective_url_base": URL_BASE,
        "generated_at": datetime.now(timezone.utc).isoformat(),
    }

    context = {
        "role": "PM_AI",
        "shared_constitution": shared,
        "role_constitution": pm,
        "policies": policies,
        "manifest": manifest,
        "status_language": status_language,
        "authority_order": [
            "Human final promotion authority",
            "Arqon Zero Shared GPT Constitution",
            "PM AI GPT Constitution",
            "Governance policies",
            "Human task request",
            "Project context",
            "PM convenience or preference",
        ],
        "fallback_policy": {
            "live_load_required_for_governance_sensitive_tasks": True,
            "stale_fallback_allowed": True,
            "stale_fallback_status": "REQUIRES_STALENESS_REVIEW",
        },
        "role_boundaries": {
            "allowed": [
                "define objectives",
                "define scope and non-scope",
                "define acceptance criteria before execution",
                "define gate plans",
                "define evidence requirements",
                "define risk registers",
                "define rollback and quarantine rules",
                "define Coder AI task packets",
                "define Helper/Codex execution packets",
                "define Auditor AI audit requests",
                "propose constitutional amendments",
                "classify work routing",
            ],
            "forbidden": [
                "write implementation code for own spec",
                "apply patches",
                "run final authoritative evidence",
                "audit own spec as final verifier",
                "certify results",
                "promote release",
                "relax gates after seeing results",
                "change acceptance criteria after execution",
                "access sealed or holdout contents",
                "modify sealed or holdout fixtures",
                "convert diagnostic evidence into promotion claims",
                "modify Constitution outside amendment protocol",
            ],
        },
    }

    context_hash = write_json("pm_gpt_context.json", context)
    print("Wrote shared/constitution/pm_gpt_context.json")
    print(f"Context hash: {context_hash}")

    # Mirror to docs for GitHub Pages publication
    docs_mirror = ROOT / "docs" / "docs" / "shared" / "constitution"
    if (ROOT / "docs" / "docs").exists():
        import shutil
        docs_mirror.mkdir(parents=True, exist_ok=True)
        for f in CONSTITUTION_DIR.glob("*"):
            if f.is_file():
                shutil.copy2(f, docs_mirror / f.name)
        print(f"Mirrored constitution files to {docs_mirror}")


if __name__ == "__main__":
    main()