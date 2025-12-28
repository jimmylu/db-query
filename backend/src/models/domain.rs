use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Domain represents an organizational unit for grouping database connections, queries, and history.
/// Provides complete data isolation between different domains (e.g., Production, Development, Analytics).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Domain {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Domain {
    /// Create a new domain with a generated UUID and current timestamp
    pub fn new(name: String, description: Option<String>) -> Result<Self, String> {
        // Validate name before creating
        Self::validate_name(&name)?;
        Self::validate_description(&description)?;

        Ok(Self {
            id: Uuid::new_v4().to_string(),
            name,
            description,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }

    /// Validate domain name: 1-50 characters, alphanumeric with spaces, hyphens, underscores
    pub fn validate_name(name: &str) -> Result<(), String> {
        let trimmed = name.trim();

        if trimmed.is_empty() {
            return Err("Domain name cannot be empty".to_string());
        }

        if trimmed.len() > 50 {
            return Err(format!(
                "Domain name cannot exceed 50 characters (got {})",
                trimmed.len()
            ));
        }

        // Check for valid characters: alphanumeric, spaces, hyphens, underscores
        if !trimmed.chars().all(|c| c.is_alphanumeric() || c == ' ' || c == '-' || c == '_') {
            return Err("Domain name contains invalid characters. Only alphanumeric characters, spaces, hyphens, and underscores are allowed".to_string());
        }

        Ok(())
    }

    /// Validate description: optional, max 500 characters
    pub fn validate_description(description: &Option<String>) -> Result<(), String> {
        if let Some(desc) = description {
            if desc.len() > 500 {
                return Err(format!(
                    "Description cannot exceed 500 characters (got {})",
                    desc.len()
                ));
            }
        }
        Ok(())
    }

    /// Update the updated_at timestamp
    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
    }
}

/// Request payload for creating a new domain
#[derive(Debug, Deserialize)]
pub struct CreateDomainRequest {
    pub name: String,
    pub description: Option<String>,
}

/// Request payload for updating an existing domain
#[derive(Debug, Deserialize)]
pub struct UpdateDomainRequest {
    pub name: Option<String>,
    pub description: Option<String>,
}

/// Response payload for domain with resource counts
#[derive(Debug, Serialize)]
pub struct DomainResponse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub connection_count: usize,
    pub saved_query_count: usize,
    pub query_history_count: usize,
}

impl From<Domain> for DomainResponse {
    fn from(domain: Domain) -> Self {
        Self {
            id: domain.id,
            name: domain.name,
            description: domain.description,
            created_at: domain.created_at,
            updated_at: domain.updated_at,
            connection_count: 0,
            saved_query_count: 0,
            query_history_count: 0,
        }
    }
}

impl DomainResponse {
    /// Create a DomainResponse with resource counts
    pub fn with_counts(
        domain: Domain,
        connection_count: usize,
        saved_query_count: usize,
        query_history_count: usize,
    ) -> Self {
        Self {
            id: domain.id,
            name: domain.name,
            description: domain.description,
            created_at: domain.created_at,
            updated_at: domain.updated_at,
            connection_count,
            saved_query_count,
            query_history_count,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_creation() {
        let domain = Domain::new("Production".to_string(), Some("Prod environment".to_string()))
            .expect("Should create domain");

        assert_eq!(domain.name, "Production");
        assert_eq!(domain.description, Some("Prod environment".to_string()));
        assert!(!domain.id.is_empty());
        assert!(domain.id.len() == 36); // UUID v4 with hyphens
    }

    #[test]
    fn test_validate_name_empty() {
        assert!(Domain::validate_name("").is_err());
        assert!(Domain::validate_name("   ").is_err());
    }

    #[test]
    fn test_validate_name_too_long() {
        let long_name = "a".repeat(51);
        assert!(Domain::validate_name(&long_name).is_err());
    }

    #[test]
    fn test_validate_name_invalid_characters() {
        assert!(Domain::validate_name("Test@Domain").is_err());
        assert!(Domain::validate_name("Test#123").is_err());
        assert!(Domain::validate_name("Test/Domain").is_err());
    }

    #[test]
    fn test_validate_name_valid() {
        assert!(Domain::validate_name("Production").is_ok());
        assert!(Domain::validate_name("Test Environment").is_ok());
        assert!(Domain::validate_name("Dev-123").is_ok());
        assert!(Domain::validate_name("Analytics_DB").is_ok());
        assert!(Domain::validate_name("Test-Env_123").is_ok());
    }

    #[test]
    fn test_validate_description() {
        assert!(Domain::validate_description(&None).is_ok());
        assert!(Domain::validate_description(&Some("Short desc".to_string())).is_ok());

        let long_desc = "a".repeat(501);
        assert!(Domain::validate_description(&Some(long_desc)).is_err());
    }

    #[test]
    fn test_domain_touch() {
        let mut domain = Domain::new("Test".to_string(), None).unwrap();
        let original_updated = domain.updated_at;

        std::thread::sleep(std::time::Duration::from_millis(10));
        domain.touch();

        assert!(domain.updated_at > original_updated);
    }
}
