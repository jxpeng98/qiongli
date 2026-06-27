from __future__ import annotations

from dataclasses import dataclass
from typing import Any

from .project_manifest import ProjectManifestState


SUBJECT_TO_DOMAIN = {
    "auto": "auto",
    "core": "auto",
    "economics": "economics",
    "accounting": "accounting",
    "business": "business-management",
    "finance": "finance",
    "political-economy": "political-economy",
    "geoeconomics": "geoeconomics",
    "economics-accounting": "economics",
}


@dataclass(frozen=True)
class ProjectSubjectState:
    effective_subject: str
    domain: str
    domain_source: str
    venue_profiles: list[str]
    method_lenses: list[str]
    strictness: str
    summary: str

    def to_packet(self) -> dict[str, Any]:
        return {
            "effective_subject": self.effective_subject,
            "domain": self.domain,
            "domain_source": self.domain_source,
            "venue_profiles": list(self.venue_profiles),
            "method_lenses": list(self.method_lenses),
            "strictness": self.strictness,
            "summary": self.summary,
        }


def resolve_project_subject(
    manifest_state: ProjectManifestState,
    *,
    requested_domain: str | None,
) -> ProjectSubjectState:
    manifest = manifest_state.manifest
    subject = manifest.active_subject
    requested = str(requested_domain or "auto").strip().lower() or "auto"
    if requested != "auto":
        domain = requested
        domain_source = "task-argument"
    else:
        domain = SUBJECT_TO_DOMAIN.get(subject, "auto")
        domain_source = (
            "project-manifest"
            if manifest_state.exists and subject not in {"auto", "core"}
            else "auto"
        )
    summary = (
        f"Project subject context: effective_subject={subject}; "
        f"domain={domain}; domain_source={domain_source}; "
        f"venue_profiles={', '.join(manifest.venue_profiles or []) or 'none'}; "
        f"method_lenses={', '.join(manifest.method_lenses or []) or 'none'}; "
        f"strictness={manifest.strictness}."
    )
    return ProjectSubjectState(
        effective_subject=subject,
        domain=domain,
        domain_source=domain_source,
        venue_profiles=list(manifest.venue_profiles or []),
        method_lenses=list(manifest.method_lenses or []),
        strictness=manifest.strictness,
        summary=summary,
    )
