use anyhow::Result;

/// Validate that a query is bike-related
pub struct QueryValidator {
    bike_keywords: Vec<String>,
}

impl QueryValidator {
    pub fn new() -> Self {
        Self {
            bike_keywords: vec![
                // General bike terms
                "motorcycle", "bike", "motorbike", "scooter", "moped",
                
                // Repair & maintenance
                "repair", "fix", "maintenance", "service", "tune", "adjust",
                "replace", "install", "remove", "clean", "inspect", "check",
                
                // Engine components
                "engine", "motor", "piston", "cylinder", "crankshaft",
                "camshaft", "valve", "timing", "compression", "carburetor",
                "fuel injection", "throttle", "choke",
                
                // Electrical
                "battery", "spark plug", "ignition", "starter", "alternator",
                "wiring", "fuse", "headlight", "taillight", "electrical",
                
                // Drivetrain
                "clutch", "transmission", "gearbox", "chain", "sprocket",
                "drive belt", "gear", "shift",
                
                // Suspension & brakes
                "fork", "suspension", "shock", "brake", "caliper",
                "disc", "pad", "fluid", "master cylinder",
                
                // Wheels & tires
                "tire", "tyre", "wheel", "rim", "spoke", "tube",
                
                // Fuel system
                "fuel", "gas", "petrol", "tank", "carburetor", "injector",
                "filter",
                
                // Exhaust
                "exhaust", "muffler", "header", "pipe", "catalytic",
                
                // Cooling
                "coolant", "radiator", "cooling", "thermostat",
                
                // Lubrication
                "oil", "lubricant", "grease",
                
                // Popular bike models
                "honda", "yamaha", "kawasaki", "suzuki", "ducati",
                "harley", "bmw", "ktm", "triumph", "aprilia",
                "cbr", "r1", "r6", "ninja", "gsxr", "zx",
            ]
            .iter()
            .map(|s| s.to_lowercase())
            .collect(),
        }
    }

    /// Validate a query for bike-related content
    pub fn validate(&self, query: &str) -> Result<()> {
        // Basic validation
        if query.trim().is_empty() {
            anyhow::bail!("Query cannot be empty");
        }

        if query.len() > 1000 {
            anyhow::bail!("Query is too long (max 1000 characters)");
        }

        // Check for malicious patterns
        self.check_malicious_patterns(query)?;

        // Check for bike-related keywords
        let query_lower = query.to_lowercase();
        let has_bike_keyword = self.bike_keywords
            .iter()
            .any(|keyword| query_lower.contains(keyword));

        if !has_bike_keyword {
            anyhow::bail!(
                "This chatbot only answers motorcycle repair and maintenance questions. \
                Your query doesn't appear to be bike-related."
            );
        }

        Ok(())
    }

    /// Check for SQL injection, XSS, and other malicious patterns
    fn check_malicious_patterns(&self, query: &str) -> Result<()> {
        let dangerous_patterns = [
            "drop table",
            "delete from",
            "insert into",
            "update set",
            "<script",
            "javascript:",
            "onerror=",
            "onclick=",
            "../",
            "..\\",
        ];

        let query_lower = query.to_lowercase();
        for pattern in &dangerous_patterns {
            if query_lower.contains(pattern) {
                log::warn!("Blocked malicious query pattern: {}", pattern);
                anyhow::bail!("Query contains invalid characters or patterns");
            }
        }

        // Check for excessive special characters (possible injection)
        let special_char_count = query.chars()
            .filter(|c| !c.is_alphanumeric() && !c.is_whitespace() && *c != '?' && *c != '.' && *c != ',' && *c != '\'' && *c != '-')
            .count();

        if special_char_count as f32 / query.len() as f32 > 0.3 {
            log::warn!("Blocked query with excessive special characters");
            anyhow::bail!("Query contains too many special characters");
        }

        Ok(())
    }

    /// Add custom bike keywords (for extensibility)
    pub fn add_keyword(&mut self, keyword: impl Into<String>) {
        self.bike_keywords.push(keyword.into().to_lowercase());
    }
}

impl Default for QueryValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_bike_queries() {
        let validator = QueryValidator::new();
        
        assert!(validator.validate("How do I change my motorcycle oil?").is_ok());
        assert!(validator.validate("Honda CBR600RR brake maintenance").is_ok());
        assert!(validator.validate("Why is my bike engine making noise?").is_ok());
    }

    #[test]
    fn test_invalid_non_bike_queries() {
        let validator = QueryValidator::new();
        
        assert!(validator.validate("What's the weather today?").is_err());
        assert!(validator.validate("Tell me a joke").is_err());
        assert!(validator.validate("Who won the game?").is_err());
    }

    #[test]
    fn test_malicious_queries() {
        let validator = QueryValidator::new();
        
        assert!(validator.validate("DROP TABLE users").is_err());
        assert!(validator.validate("<script>alert('xss')</script>").is_err());
        assert!(validator.validate("../../../etc/passwd").is_err());
    }

    #[test]
    fn test_empty_query() {
        let validator = QueryValidator::new();
        assert!(validator.validate("").is_err());
        assert!(validator.validate("   ").is_err());
    }
}
