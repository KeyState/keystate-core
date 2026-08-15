//! The verification output shapes: [`Severity`], [`IssueKind`],
//! [`VerificationIssue`] and [`VerificationReport`].

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::Version;
use crate::model::EntityKind;

/// Severity of a single verification issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    /// A real completeness gap: required config state is missing.
    Error,
    /// Tracked state that falls short but does not block the completeness
    /// claim (e.g. a volatile field missing purely due to null session data).
    Warning,
}

/// The machine-readable category of a verification issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IssueKind {
    /// A `Required` entity type has no rows.
    MissingEntity,
    /// A required field is absent from an existing row.
    MissingRequired,
    /// A non-required field is absent from every row of its entity.
    MissingOptional,
    /// A manifest-volatile field leaked into the config body.
    MisplacedVolatile,
    /// A manifest-volatile field was not captured in the volatile section.
    MissingVolatile,
}

/// A single finding from verification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerificationIssue {
    /// Severity of the finding.
    pub severity: Severity,
    /// Category of the finding, for machine consumers.
    pub kind: IssueKind,
    /// The entity kind the finding concerns.
    pub entity: EntityKind,
    /// Row id the finding concerns, when row-scoped.
    pub row_id: Option<String>,
    /// The field path the finding concerns, when field-scoped.
    pub path: Option<String>,
    /// Schema version that introduced the absent field, when known.
    pub introduced_in: Option<Version>,
    /// Human-readable explanation.
    pub message: String,
}

impl VerificationIssue {
    /// A concise, human-readable one-liner.
    pub fn summary(&self) -> String {
        match (&self.row_id, &self.path) {
            (Some(row_id), Some(path)) => {
                format!(
                    "{} [{}/{}] {} -> {}",
                    self.kind_label(),
                    self.entity_label(),
                    row_id,
                    path,
                    self.message
                )
            }
            (Some(row_id), None) => {
                format!(
                    "{} [{}/{}] {}",
                    self.kind_label(),
                    self.entity_label(),
                    row_id,
                    self.message
                )
            }
            _ => format!(
                "{} [{}] {}",
                self.kind_label(),
                self.entity_label(),
                self.message
            ),
        }
    }

    fn kind_label(&self) -> &'static str {
        match self.kind {
            IssueKind::MissingEntity => "missing-entity",
            IssueKind::MissingRequired => "missing-required",
            IssueKind::MissingOptional => "missing-optional",
            IssueKind::MisplacedVolatile => "misplaced-volatile",
            IssueKind::MissingVolatile => "missing-volatile",
        }
    }

    fn entity_label(&self) -> &'static str {
        super::entity_label(self.entity)
    }
}

/// The complete result of running a manifest against an extraction.
///
/// Part of the output document, not a side log: completeness is a claim the
/// tool backs with evidence every time it runs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerificationReport {
    /// Backend the report applies to.
    pub backend: String,
    /// Backend schema version the manifest was keyed on.
    pub against_version: Version,
    /// All findings, in manifest order.
    pub issues: Vec<VerificationIssue>,
}

impl VerificationReport {
    /// Whether the extraction is complete: no `Error`-severity findings.
    pub fn is_complete(&self) -> bool {
        self.issues.iter().all(|i| i.severity != Severity::Error)
    }

    /// Counts of `(errors, warnings)`.
    pub fn counts(&self) -> (usize, usize) {
        let errors = self
            .issues
            .iter()
            .filter(|i| i.severity == Severity::Error)
            .count();
        (errors, self.issues.len() - errors)
    }
}

impl fmt::Display for VerificationReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (errors, warnings) = self.counts();
        write!(
            f,
            "VerificationReport({} @ {}, {} errors, {} warnings, {} issues)",
            self.backend,
            self.against_version,
            errors,
            warnings,
            self.issues.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn issue(kind: IssueKind, severity: Severity) -> VerificationIssue {
        VerificationIssue {
            severity,
            kind,
            entity: EntityKind::Client,
            row_id: Some("c1".into()),
            path: Some("native.secret".into()),
            introduced_in: Some(Version::new(1, 0, 0)),
            message: "boom".into(),
        }
    }

    #[test]
    fn report_counts_and_completeness() {
        let report = VerificationReport {
            backend: "stub".into(),
            against_version: Version::new(1, 0, 0),
            issues: vec![
                issue(IssueKind::MissingRequired, Severity::Error),
                issue(IssueKind::MissingVolatile, Severity::Warning),
            ],
        };
        assert!(!report.is_complete());
        assert_eq!(report.counts(), (1, 1));

        let clean = VerificationReport {
            issues: vec![],
            ..report.clone()
        };
        assert!(clean.is_complete());
        assert_eq!(clean.counts(), (0, 0));
    }

    #[test]
    fn issue_summary_includes_entity_and_path() {
        let s = issue(IssueKind::MissingRequired, Severity::Error).summary();
        assert!(s.contains("missing-required"));
        assert!(s.contains("client"));
        assert!(s.contains("native.secret"));
    }
}
