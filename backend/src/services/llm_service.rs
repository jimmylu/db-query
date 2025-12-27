use crate::models::DatabaseMetadata;
use crate::api::middleware::AppError;
use crate::config::Config;
use serde_json::json;
use reqwest::Client as HttpClient;

/// LLM service for converting metadata to JSON format and generating SQL from natural language
pub struct LlmService {
    gateway_url: String,
    api_key: Option<String>,
    http_client: HttpClient,
}

impl LlmService {
    pub fn new(config: &Config) -> Self {
        Self {
            gateway_url: config.llm.gateway_url.clone(),
            api_key: config.llm.api_key.clone(),
            http_client: HttpClient::new(),
        }
    }

    /// Convert metadata to JSON format using LLM
    /// For Phase 3, we'll use a simple JSON serialization
    /// Full LLM integration will be added when rig.rs is available
    pub async fn convert_metadata_to_json(
        &self,
        metadata: &DatabaseMetadata,
    ) -> Result<String, AppError> {
        // For Phase 3, we'll use direct JSON serialization
        // Full LLM processing will be implemented when rig.rs is integrated
        
        let json_value = json!({
            "tables": metadata.tables,
            "views": metadata.views,
            "schemas": metadata.schemas,
            "retrieved_at": metadata.retrieved_at.to_rfc3339(),
        });

        serde_json::to_string(&json_value)
            .map_err(|e| AppError::LlmService(format!("Failed to serialize metadata: {}", e)))
    }

    /// Prepare metadata context for LLM
    pub fn prepare_metadata_context(&self, metadata: &DatabaseMetadata) -> String {
        let mut context = String::from("Database Schema:\n\n");
        
        // Add schemas
        if !metadata.schemas.is_empty() {
            context.push_str("Schemas: ");
            context.push_str(&metadata.schemas.join(", "));
            context.push_str("\n\n");
        }
        
        // Add tables
        if !metadata.tables.is_empty() {
            context.push_str("Tables:\n");
            for table in &metadata.tables {
                context.push_str(&format!("  - {}.{}\n", 
                    table.schema.as_ref().unwrap_or(&"public".to_string()),
                    table.name
                ));
                context.push_str("    Columns:\n");
                for column in &table.columns {
                    context.push_str(&format!("      * {} ({})", column.name, column.data_type));
                    if column.is_primary_key {
                        context.push_str(" [PRIMARY KEY]");
                    }
                    if column.is_foreign_key {
                        context.push_str(" [FOREIGN KEY]");
                    }
                    if !column.is_nullable {
                        context.push_str(" [NOT NULL]");
                    }
                    context.push('\n');
                }
            }
            context.push('\n');
        }
        
        // Add views
        if !metadata.views.is_empty() {
            context.push_str("Views:\n");
            for view in &metadata.views {
                context.push_str(&format!("  - {}.{}\n",
                    view.schema.as_ref().unwrap_or(&"public".to_string()),
                    view.name
                ));
                context.push_str("    Columns:\n");
                for column in &view.columns {
                    context.push_str(&format!("      * {} ({})\n", column.name, column.data_type));
                }
            }
        }
        
        context
    }

    /// Generate SQL query from natural language
    pub async fn generate_sql_from_natural_language(
        &self,
        question: &str,
        metadata: &DatabaseMetadata,
        database_type: &str,  // Add database_type parameter
    ) -> Result<String, AppError> {
        // Prepare metadata context
        let metadata_context = self.prepare_metadata_context(metadata);

        // Determine SQL dialect hints based on database type
        let dialect_hints = match database_type {
            "mysql" => r#"
- Use MySQL syntax and functions
- Use LIMIT syntax (not TOP or FETCH FIRST)
- For dates, use functions like NOW(), CURDATE(), DATE_SUB(), etc.
- String concatenation uses CONCAT() function
- Use backticks for identifier quoting if needed: `table_name`"#,
            "postgresql" | _ => r#"
- Use PostgreSQL syntax and functions
- Use LIMIT syntax (or FETCH FIRST)
- For dates, use functions like NOW(), CURRENT_DATE, interval arithmetic
- String concatenation uses || operator or CONCAT()
- Use double quotes for identifier quoting if needed: "table_name""#,
        };

        // Create prompt for LLM
        let prompt = format!(
            r#"You are a SQL expert. Given a database schema and a natural language question, generate a valid {database_type} SELECT query.

Database Schema:
{metadata_context}

Question: {question}

Instructions:
1. Generate ONLY a valid {database_type} SELECT query
2. Do not include any explanations or markdown formatting
3. Use proper table and column names from the schema above
4. Return ONLY the SQL query, nothing else
5. If the question asks about "数量" (count) or "多少" (how many), use COUNT(*)
6. If the question asks about specific columns, select only those columns
{dialect_hints}

SQL Query:"#,
            database_type = database_type,
            metadata_context = metadata_context,
            question = question,
            dialect_hints = dialect_hints
        );

        // Call LLM service
        // For now, we'll use a simple HTTP-based approach
        // In production, this would use rig.rs or a proper LLM gateway
        self.call_llm_api(&prompt).await
    }

    /// Call LLM API to generate SQL
    async fn call_llm_api(&self, prompt: &str) -> Result<String, AppError> {
        // Check if LLM gateway is configured
        if self.gateway_url.is_empty() || self.gateway_url == "http://localhost:8080" {
            // Fallback: Use a simple rule-based approach for demonstration
            return self.fallback_sql_generation(prompt);
        }

        // Prepare request
        let mut request = self.http_client
            .post(&self.gateway_url)
            .json(&json!({
                "prompt": prompt,
                "max_tokens": 500,
                "temperature": 0.1,
            }));

        // Add API key if available
        if let Some(api_key) = &self.api_key {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }

        // Send request
        let response = request
            .send()
            .await
            .map_err(|e| AppError::LlmService(format!("Failed to call LLM service: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AppError::LlmService(format!(
                "LLM service returned error {}: {}",
                status, error_text
            )));
        }

        // Parse response
        let result: serde_json::Value = response
            .json()
            .await
            .map_err(|e| AppError::LlmService(format!("Failed to parse LLM response: {}", e)))?;

        // Extract SQL from response (adjust based on your LLM API format)
        let sql = result["text"]
            .as_str()
            .or_else(|| result["content"].as_str())
            .or_else(|| result["response"].as_str())
            .ok_or_else(|| {
                AppError::LlmService("LLM response does not contain SQL query".to_string())
            })?;

        // Clean up SQL (remove markdown code blocks if present)
        let cleaned_sql = sql
            .trim()
            .trim_start_matches("```sql")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim()
            .to_string();

        Ok(cleaned_sql)
    }

    /// Fallback SQL generation using simple pattern matching
    /// This is used when LLM service is not configured
    fn fallback_sql_generation(&self, prompt: &str) -> Result<String, AppError> {
        // Extract question from prompt
        let question = if let Some(start) = prompt.find("Question:") {
            prompt[start + 9..].trim()
        } else if let Some(start) = prompt.find("question:") {
            prompt[start + 9..].trim()
        } else {
            prompt.trim()
        };

        // Try to extract table name from metadata context in the prompt
        // Look for table names in the schema section
        let mut table_name: Option<String> = None;
        
        // First, try to find table names from the schema context
        if let Some(schema_start) = prompt.find("Tables:") {
            let schema_section = &prompt[schema_start..];
            // Look for table patterns like "  - schema.table_name" or "  - table_name"
            for line in schema_section.lines() {
                if line.trim().starts_with("- ") {
                    let table_line = line.trim_start_matches("- ").trim();
                    // Extract table name (format: schema.table or just table)
                    if let Some(dot_pos) = table_line.find('.') {
                        let name = &table_line[dot_pos + 1..];
                        if !name.is_empty() && name.len() < 100 && !name.contains(' ') {
                            // Valid table name found
                            table_name = Some(name.to_string());
                            break;
                        }
                    } else if !table_line.is_empty() && table_line.len() < 100 && !table_line.contains(' ') {
                        // No schema, just table name
                        table_name = Some(table_line.to_string());
                        break;
                    }
                }
            }
        }
        
        // If no table found in schema, try to extract from question using common patterns
        if table_name.is_none() {
            // Common Chinese patterns for table names
            let table_keywords = [
                ("公司", "companies"),
                ("用户", "users"),
                ("订单", "orders"),
                ("产品", "products"),
                ("客户", "customers"),
                ("员工", "employees"),
                ("聊天", "chats"),
                ("消息", "messages"),
                ("记录", "records"),
                ("聊天记录", "chats"),
            ];
            
            for (chinese, english) in table_keywords.iter() {
                if question.contains(chinese) {
                    table_name = Some(english.to_string());
                    break;
                }
            }
        }
        
        // If still no table found, try to match against actual tables in metadata
        // This requires parsing the metadata from the prompt
        if table_name.is_none() {
            // Try to find the first table mentioned in the schema
            if let Some(schema_start) = prompt.find("Tables:") {
                let schema_section = &prompt[schema_start..];
                // Get first table name
                for line in schema_section.lines() {
                    if line.trim().starts_with("- ") {
                        let table_line = line.trim_start_matches("- ").trim();
                        if let Some(dot_pos) = table_line.find('.') {
                            let name = &table_line[dot_pos + 1..];
                            if !name.is_empty() && name.len() < 100 && !name.contains(' ') {
                                table_name = Some(name.to_string());
                                break;
                            }
                        } else if !table_line.is_empty() && table_line.len() < 100 && !table_line.contains(' ') {
                            table_name = Some(table_line.to_string());
                            break;
                        }
                    }
                }
            }
        }
        
        // Default fallback - use first available table or a safe default
        let table_name = table_name.unwrap_or_else(|| {
            // If we have metadata, try to get first table
            if let Some(schema_start) = prompt.find("Tables:") {
                let schema_section = &prompt[schema_start..];
                for line in schema_section.lines() {
                    if line.trim().starts_with("- ") {
                        let table_line = line.trim_start_matches("- ").trim();
                        if let Some(dot_pos) = table_line.find('.') {
                            return table_line[dot_pos + 1..].to_string();
                        } else if !table_line.is_empty() {
                            return table_line.to_string();
                        }
                    }
                }
            }
            "companies".to_string()
        });
        
        // Check if question asks for count
        let is_count_query = question.contains("数量") || question.contains("多少") || 
                            question.contains("count") || question.contains("how many");
        
        // Generate SQL query
        let sql = if is_count_query {
            format!("SELECT COUNT(*) FROM {}", table_name)
        } else {
            format!("SELECT * FROM {}", table_name)
        };
        
        tracing::warn!("Using fallback SQL generation. Configure LLM service for better results. Generated: {}", sql);
        tracing::warn!("Question was: {}", question);
        
        Ok(sql)
    }
}

