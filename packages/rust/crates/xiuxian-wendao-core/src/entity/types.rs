//! Entity and relation type labels used by extracted knowledge graphs.

use std::convert::Infallible;
use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// Predefined entity categories for the knowledge graph.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntityType {
    /// Person or character.
    Person,
    /// Geographic location or place.
    Location,
    /// Organization, group, or sect.
    Organization,
    /// Cultivation technique, skill, or manual.
    Technique,
    /// Spiritual item, weapon, or material.
    Artifact,
    /// Concept, law, or abstract idea.
    Concept,
    /// Historical event or incident.
    Event,
    /// Document or source file.
    Document,
    /// Skill descriptor.
    Skill,
    /// Project.
    Project,
    /// Tool.
    Tool,
    /// Code.
    Code,
    /// API.
    Api,
    /// Error.
    Error,
    /// Pattern.
    Pattern,
    /// Other uncategorized entity.
    Other(String),
}

impl fmt::Display for EntityType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Person => write!(f, "PERSON"),
            Self::Location => write!(f, "LOCATION"),
            Self::Organization => write!(f, "ORGANIZATION"),
            Self::Technique => write!(f, "TECHNIQUE"),
            Self::Artifact => write!(f, "ARTIFACT"),
            Self::Concept => write!(f, "CONCEPT"),
            Self::Event => write!(f, "EVENT"),
            Self::Document => write!(f, "DOCUMENT"),
            Self::Skill => write!(f, "SKILL"),
            Self::Project => write!(f, "PROJECT"),
            Self::Tool => write!(f, "TOOL"),
            Self::Code => write!(f, "CODE"),
            Self::Api => write!(f, "API"),
            Self::Error => write!(f, "ERROR"),
            Self::Pattern => write!(f, "PATTERN"),
            Self::Other(s) => write!(f, "{}", s.to_uppercase()),
        }
    }
}

impl EntityType {
    fn parse_lossy(raw: &str) -> Self {
        match raw.to_uppercase().as_str() {
            "PERSON" => Self::Person,
            "LOCATION" => Self::Location,
            "ORGANIZATION" => Self::Organization,
            "TECHNIQUE" => Self::Technique,
            "ARTIFACT" => Self::Artifact,
            "CONCEPT" => Self::Concept,
            "EVENT" => Self::Event,
            "DOCUMENT" => Self::Document,
            "SKILL" => Self::Skill,
            "PROJECT" => Self::Project,
            "TOOL" => Self::Tool,
            "CODE" => Self::Code,
            "API" => Self::Api,
            "ERROR" => Self::Error,
            "PATTERN" => Self::Pattern,
            _ => Self::Other(raw.to_string()),
        }
    }
}

impl FromStr for EntityType {
    type Err = Infallible;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        Ok(Self::parse_lossy(raw))
    }
}

/// Predefined relation types for the knowledge graph.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationType {
    /// Entity is located in another entity.
    LocatedIn,
    /// Entity is a member of an organization.
    MemberOf,
    /// Entity created another entity.
    CreatedBy,
    /// Entity is documented in a source.
    DocumentedIn,
    /// Generic related relationship.
    RelatedTo,
    /// Entity implements a technique.
    Implements,
    /// Entity extends or is a child of another.
    Extends,
    /// Entity contains another.
    Contains,
    /// Skill governs an intent.
    Governs,
    /// Works for.
    WorksFor,
    /// Part of.
    PartOf,
    /// Uses.
    Uses,
    /// Depends on.
    DependsOn,
    /// Similar to.
    SimilarTo,
    /// References.
    References,
    /// Manifests.
    Manifests,
    /// Attached to.
    AttachedTo,
    /// Other custom relation.
    Other(String),
}

impl fmt::Display for RelationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LocatedIn => write!(f, "LOCATED_IN"),
            Self::MemberOf => write!(f, "MEMBER_OF"),
            Self::CreatedBy => write!(f, "CREATED_BY"),
            Self::DocumentedIn => write!(f, "DOCUMENTED_IN"),
            Self::RelatedTo => write!(f, "RELATED_TO"),
            Self::Implements => write!(f, "IMPLEMENTS"),
            Self::Extends => write!(f, "EXTENDS"),
            Self::Contains => write!(f, "CONTAINS"),
            Self::Governs => write!(f, "GOVERNS"),
            Self::WorksFor => write!(f, "WORKS_FOR"),
            Self::PartOf => write!(f, "PART_OF"),
            Self::Uses => write!(f, "USES"),
            Self::DependsOn => write!(f, "DEPENDS_ON"),
            Self::SimilarTo => write!(f, "SIMILAR_TO"),
            Self::References => write!(f, "REFERENCES"),
            Self::Manifests => write!(f, "MANIFESTS"),
            Self::AttachedTo => write!(f, "ATTACHED_TO"),
            Self::Other(s) => write!(f, "{}", s.to_uppercase()),
        }
    }
}

impl RelationType {
    fn parse_lossy(raw: &str) -> Self {
        match raw.to_uppercase().as_str() {
            "LOCATED_IN" => Self::LocatedIn,
            "MEMBER_OF" => Self::MemberOf,
            "CREATED_BY" => Self::CreatedBy,
            "DOCUMENTED_IN" => Self::DocumentedIn,
            "RELATED_TO" => Self::RelatedTo,
            "IMPLEMENTS" => Self::Implements,
            "EXTENDS" => Self::Extends,
            "CONTAINS" => Self::Contains,
            "GOVERNS" => Self::Governs,
            "WORKS_FOR" => Self::WorksFor,
            "PART_OF" => Self::PartOf,
            "USES" => Self::Uses,
            "DEPENDS_ON" => Self::DependsOn,
            "SIMILAR_TO" => Self::SimilarTo,
            "REFERENCES" => Self::References,
            "MANIFESTS" => Self::Manifests,
            "ATTACHED_TO" => Self::AttachedTo,
            _ => Self::Other(raw.to_string()),
        }
    }
}

impl FromStr for RelationType {
    type Err = Infallible;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        Ok(Self::parse_lossy(raw))
    }
}
