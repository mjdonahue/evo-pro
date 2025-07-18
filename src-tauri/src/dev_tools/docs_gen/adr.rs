//! Architecture Decision Records (ADRs)
//!
//! This module provides tools for creating, managing, and generating documentation
//! for Architecture Decision Records (ADRs). ADRs are documents that capture important
//! architectural decisions made along with their context and consequences.

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use serde::{Serialize, Deserialize};
use tracing::{debug, info, warn, error};

use super::DocsGenConfig;

/// ADR status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AdrStatus {
    /// Proposed but not yet accepted
    Proposed,
    /// Accepted and active
    Accepted,
    /// Rejected
    Rejected,
    /// Deprecated but still relevant for historical purposes
    Deprecated,
    /// Superseded by another ADR
    Superseded,
    /// Amended by another ADR
    Amended,
}

impl std::fmt::Display for AdrStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AdrStatus::Proposed => write!(f, "Proposed"),
            AdrStatus::Accepted => write!(f, "Accepted"),
            AdrStatus::Rejected => write!(f, "Rejected"),
            AdrStatus::Deprecated => write!(f, "Deprecated"),
            AdrStatus::Superseded => write!(f, "Superseded"),
            AdrStatus::Amended => write!(f, "Amended"),
        }
    }
}

/// Architecture Decision Record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitectureDecisionRecord {
    /// ADR ID (e.g., "ADR-001")
    pub id: String,
    /// ADR title
    pub title: String,
    /// ADR status
    pub status: AdrStatus,
    /// Date when the ADR was created
    pub date: String,
    /// Authors of the ADR
    pub authors: Vec<String>,
    /// Technical leads who approved the ADR
    pub approvers: Vec<String>,
    /// Context and problem statement
    pub context: String,
    /// Decision made
    pub decision: String,
    /// Consequences of the decision
    pub consequences: String,
    /// Alternatives considered
    pub alternatives: Vec<String>,
    /// Related ADRs
    pub related_adrs: Vec<String>,
    /// References and additional resources
    pub references: Vec<String>,
    /// Tags for categorization
    pub tags: Vec<String>,
    /// Superseded by this ADR (if applicable)
    pub superseded_by: Option<String>,
    /// Amended by these ADRs (if applicable)
    pub amended_by: Vec<String>,
    /// Supersedes these ADRs (if applicable)
    pub supersedes: Vec<String>,
    /// Amends these ADRs (if applicable)
    pub amends: Vec<String>,
}

impl ArchitectureDecisionRecord {
    /// Create a new Architecture Decision Record
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        status: AdrStatus,
        date: impl Into<String>,
        context: impl Into<String>,
        decision: impl Into<String>,
        consequences: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            status,
            date: date.into(),
            authors: Vec::new(),
            approvers: Vec::new(),
            context: context.into(),
            decision: decision.into(),
            consequences: consequences.into(),
            alternatives: Vec::new(),
            related_adrs: Vec::new(),
            references: Vec::new(),
            tags: Vec::new(),
            superseded_by: None,
            amended_by: Vec::new(),
            supersedes: Vec::new(),
            amends: Vec::new(),
        }
    }
    
    /// Add an author to the ADR
    pub fn with_author(mut self, author: impl Into<String>) -> Self {
        self.authors.push(author.into());
        self
    }
    
    /// Add an approver to the ADR
    pub fn with_approver(mut self, approver: impl Into<String>) -> Self {
        self.approvers.push(approver.into());
        self
    }
    
    /// Add an alternative to the ADR
    pub fn with_alternative(mut self, alternative: impl Into<String>) -> Self {
        self.alternatives.push(alternative.into());
        self
    }
    
    /// Add a related ADR to the ADR
    pub fn with_related_adr(mut self, related_adr: impl Into<String>) -> Self {
        self.related_adrs.push(related_adr.into());
        self
    }
    
    /// Add a reference to the ADR
    pub fn with_reference(mut self, reference: impl Into<String>) -> Self {
        self.references.push(reference.into());
        self
    }
    
    /// Add a tag to the ADR
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }
    
    /// Set the ADR as superseded by another ADR
    pub fn superseded_by(mut self, adr_id: impl Into<String>) -> Self {
        self.superseded_by = Some(adr_id.into());
        self.status = AdrStatus::Superseded;
        self
    }
    
    /// Add an ADR that amends this ADR
    pub fn amended_by(mut self, adr_id: impl Into<String>) -> Self {
        self.amended_by.push(adr_id.into());
        self.status = AdrStatus::Amended;
        self
    }
    
    /// Add an ADR that this ADR supersedes
    pub fn supersedes(mut self, adr_id: impl Into<String>) -> Self {
        self.supersedes.push(adr_id.into());
        self
    }
    
    /// Add an ADR that this ADR amends
    pub fn amends(mut self, adr_id: impl Into<String>) -> Self {
        self.amends.push(adr_id.into());
        self
    }
    
    /// Convert the ADR to Markdown format
    pub fn to_markdown(&self) -> String {
        let mut markdown = String::new();
        
        // Title and metadata
        markdown.push_str(&format!("# {} {}\n\n", self.id, self.title));
        markdown.push_str(&format!("**Status:** {}\n\n", self.status));
        markdown.push_str(&format!("**Date:** {}\n\n", self.date));
        
        // Authors and approvers
        if !self.authors.is_empty() {
            markdown.push_str("**Authors:** ");
            markdown.push_str(&self.authors.join(", "));
            markdown.push_str("\n\n");
        }
        
        if !self.approvers.is_empty() {
            markdown.push_str("**Approvers:** ");
            markdown.push_str(&self.approvers.join(", "));
            markdown.push_str("\n\n");
        }
        
        // Tags
        if !self.tags.is_empty() {
            markdown.push_str("**Tags:** ");
            markdown.push_str(&self.tags.join(", "));
            markdown.push_str("\n\n");
        }
        
        // Related ADRs
        if !self.related_adrs.is_empty() {
            markdown.push_str("**Related ADRs:** ");
            markdown.push_str(&self.related_adrs.join(", "));
            markdown.push_str("\n\n");
        }
        
        // Supersedes/Amends relationships
        if !self.supersedes.is_empty() {
            markdown.push_str("**Supersedes:** ");
            markdown.push_str(&self.supersedes.join(", "));
            markdown.push_str("\n\n");
        }
        
        if !self.amends.is_empty() {
            markdown.push_str("**Amends:** ");
            markdown.push_str(&self.amends.join(", "));
            markdown.push_str("\n\n");
        }
        
        if let Some(superseded_by) = &self.superseded_by {
            markdown.push_str(&format!("**Superseded by:** {}\n\n", superseded_by));
        }
        
        if !self.amended_by.is_empty() {
            markdown.push_str("**Amended by:** ");
            markdown.push_str(&self.amended_by.join(", "));
            markdown.push_str("\n\n");
        }
        
        // Main content sections
        markdown.push_str("## Context\n\n");
        markdown.push_str(&self.context);
        markdown.push_str("\n\n");
        
        markdown.push_str("## Decision\n\n");
        markdown.push_str(&self.decision);
        markdown.push_str("\n\n");
        
        markdown.push_str("## Consequences\n\n");
        markdown.push_str(&self.consequences);
        markdown.push_str("\n\n");
        
        // Alternatives
        if !self.alternatives.is_empty() {
            markdown.push_str("## Alternatives Considered\n\n");
            for (i, alternative) in self.alternatives.iter().enumerate() {
                markdown.push_str(&format!("### Alternative {}\n\n", i + 1));
                markdown.push_str(alternative);
                markdown.push_str("\n\n");
            }
        }
        
        // References
        if !self.references.is_empty() {
            markdown.push_str("## References\n\n");
            for reference in &self.references {
                markdown.push_str(&format!("- {}\n", reference));
            }
        }
        
        markdown
    }
}

/// ADR configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdrConfig {
    /// Whether ADR generation is enabled
    pub enabled: bool,
    /// ADR template path
    pub template_path: Option<String>,
    /// ADR output directory
    pub output_directory: String,
    /// Whether to generate an index page
    pub generate_index: bool,
    /// Whether to generate a graph of ADR relationships
    pub generate_graph: bool,
    /// Whether to include status badges
    pub include_status_badges: bool,
    /// Additional options
    pub options: HashMap<String, String>,
}

impl Default for AdrConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            template_path: None,
            output_directory: "docs/architecture/decisions".to_string(),
            generate_index: true,
            generate_graph: true,
            include_status_badges: true,
            options: HashMap::new(),
        }
    }
}

/// ADR manager
#[derive(Debug)]
pub struct AdrManager {
    /// Configuration
    pub config: AdrConfig,
    /// Base documentation generator configuration
    pub base_config: DocsGenConfig,
    /// ADRs by ID
    pub adrs: HashMap<String, ArchitectureDecisionRecord>,
}

impl AdrManager {
    /// Create a new ADR manager
    pub fn new(config: AdrConfig, base_config: DocsGenConfig) -> Self {
        Self {
            config,
            base_config,
            adrs: HashMap::new(),
        }
    }
    
    /// Load ADRs from a directory
    pub fn load_adrs(&mut self, dir: &Path) -> io::Result<()> {
        info!("Loading ADRs from {}", dir.display());
        
        if dir.exists() && dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                
                if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                    let mut file = File::open(&path)?;
                    let mut contents = String::new();
                    file.read_to_string(&mut contents)?;
                    
                    match serde_json::from_str::<ArchitectureDecisionRecord>(&contents) {
                        Ok(adr) => {
                            self.adrs.insert(adr.id.clone(), adr);
                        }
                        Err(err) => {
                            warn!("Failed to parse ADR file {}: {}", path.display(), err);
                        }
                    }
                }
            }
        }
        
        info!("Loaded {} ADRs", self.adrs.len());
        Ok(())
    }
    
    /// Create a new ADR
    pub fn create_adr(&mut self, adr: ArchitectureDecisionRecord) -> io::Result<()> {
        // Check if ADR with this ID already exists
        if self.adrs.contains_key(&adr.id) {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!("ADR with ID {} already exists", adr.id),
            ));
        }
        
        // Add ADR to the collection
        self.adrs.insert(adr.id.clone(), adr);
        
        Ok(())
    }
    
    /// Get an ADR by ID
    pub fn get_adr(&self, id: &str) -> Option<&ArchitectureDecisionRecord> {
        self.adrs.get(id)
    }
    
    /// Update an existing ADR
    pub fn update_adr(&mut self, adr: ArchitectureDecisionRecord) -> io::Result<()> {
        // Check if ADR with this ID exists
        if !self.adrs.contains_key(&adr.id) {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("ADR with ID {} not found", adr.id),
            ));
        }
        
        // Update ADR in the collection
        self.adrs.insert(adr.id.clone(), adr);
        
        Ok(())
    }
    
    /// Delete an ADR by ID
    pub fn delete_adr(&mut self, id: &str) -> io::Result<()> {
        // Check if ADR with this ID exists
        if !self.adrs.contains_key(id) {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("ADR with ID {} not found", id),
            ));
        }
        
        // Remove ADR from the collection
        self.adrs.remove(id);
        
        Ok(())
    }
    
    /// Save ADRs to files
    pub fn save_adrs(&self, dir: &Path) -> io::Result<()> {
        info!("Saving ADRs to {}", dir.display());
        
        // Create directory if it doesn't exist
        fs::create_dir_all(dir)?;
        
        // Save each ADR as a JSON file
        for adr in self.adrs.values() {
            let file_path = dir.join(format!("{}.json", adr.id));
            let file = File::create(file_path)?;
            serde_json::to_writer_pretty(file, adr)?;
            
            // Also save as Markdown
            let md_path = dir.join(format!("{}.md", adr.id));
            let mut md_file = File::create(md_path)?;
            write!(md_file, "{}", adr.to_markdown())?;
        }
        
        info!("Saved {} ADRs", self.adrs.len());
        Ok(())
    }
    
    /// Generate ADR documentation
    pub fn generate_adr_docs(&self, output_dir: &Path) -> io::Result<()> {
        if !self.config.enabled {
            info!("ADR documentation generation is disabled");
            return Ok(());
        }
        
        info!("Generating ADR documentation");
        
        // Create output directory
        let adr_dir = output_dir.join("adr");
        fs::create_dir_all(&adr_dir)?;
        
        // Generate index page if enabled
        if self.config.generate_index {
            self.generate_index_page(&adr_dir)?;
        }
        
        // Generate ADR pages
        for adr in self.adrs.values() {
            self.generate_adr_page(&adr_dir, adr)?;
        }
        
        // Generate graph if enabled
        if self.config.generate_graph {
            self.generate_adr_graph(&adr_dir)?;
        }
        
        info!("ADR documentation generated successfully");
        Ok(())
    }
    
    /// Generate index page
    fn generate_index_page(&self, output_dir: &Path) -> io::Result<()> {
        let index_path = output_dir.join("index.html");
        let mut index_file = File::create(index_path)?;
        
        writeln!(index_file, "<!DOCTYPE html>")?;
        writeln!(index_file, "<html>")?;
        writeln!(index_file, "<head>")?;
        writeln!(index_file, "    <title>Architecture Decision Records</title>")?;
        writeln!(index_file, "    <meta charset=\"UTF-8\">")?;
        writeln!(index_file, "    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">")?;
        writeln!(index_file, "    <style>")?;
        writeln!(index_file, "        body {{ font-family: Arial, sans-serif; margin: 0; padding: 20px; }}")?;
        writeln!(index_file, "        h1, h2 {{ color: #333; }}")?;
        writeln!(index_file, "        table {{ border-collapse: collapse; width: 100%; }}")?;
        writeln!(index_file, "        th, td {{ border: 1px solid #ddd; padding: 8px; text-align: left; }}")?;
        writeln!(index_file, "        th {{ background-color: #f2f2f2; }}")?;
        writeln!(index_file, "        tr:nth-child(even) {{ background-color: #f9f9f9; }}")?;
        writeln!(index_file, "        .badge {{ display: inline-block; padding: 3px 8px; border-radius: 3px; font-size: 0.8em; color: white; }}")?;
        writeln!(index_file, "        .badge-proposed {{ background-color: #6c757d; }}")?;
        writeln!(index_file, "        .badge-accepted {{ background-color: #28a745; }}")?;
        writeln!(index_file, "        .badge-rejected {{ background-color: #dc3545; }}")?;
        writeln!(index_file, "        .badge-deprecated {{ background-color: #6c757d; }}")?;
        writeln!(index_file, "        .badge-superseded {{ background-color: #fd7e14; }}")?;
        writeln!(index_file, "        .badge-amended {{ background-color: #17a2b8; }}")?;
        writeln!(index_file, "        .tag {{ display: inline-block; background-color: #f0f0f0; padding: 2px 6px; margin-right: 5px; border-radius: 3px; font-size: 0.8em; }}")?;
        writeln!(index_file, "    </style>")?;
        writeln!(index_file, "</head>")?;
        writeln!(index_file, "<body>")?;
        writeln!(index_file, "    <h1>Architecture Decision Records</h1>")?;
        
        if self.adrs.is_empty() {
            writeln!(index_file, "    <p>No architecture decision records available.</p>")?;
        } else {
            // Group ADRs by status
            let mut proposed = Vec::new();
            let mut accepted = Vec::new();
            let mut rejected = Vec::new();
            let mut deprecated = Vec::new();
            let mut superseded = Vec::new();
            let mut amended = Vec::new();
            
            for adr in self.adrs.values() {
                match adr.status {
                    AdrStatus::Proposed => proposed.push(adr),
                    AdrStatus::Accepted => accepted.push(adr),
                    AdrStatus::Rejected => rejected.push(adr),
                    AdrStatus::Deprecated => deprecated.push(adr),
                    AdrStatus::Superseded => superseded.push(adr),
                    AdrStatus::Amended => amended.push(adr),
                }
            }
            
            // Write table of ADRs
            writeln!(index_file, "    <table>")?;
            writeln!(index_file, "        <thead>")?;
            writeln!(index_file, "            <tr>")?;
            writeln!(index_file, "                <th>ID</th>")?;
            writeln!(index_file, "                <th>Title</th>")?;
            writeln!(index_file, "                <th>Status</th>")?;
            writeln!(index_file, "                <th>Date</th>")?;
            writeln!(index_file, "                <th>Tags</th>")?;
            writeln!(index_file, "            </tr>")?;
            writeln!(index_file, "        </thead>")?;
            writeln!(index_file, "        <tbody>")?;
            
            // Write accepted ADRs first
            for adr in accepted {
                self.write_adr_table_row(&mut index_file, adr)?;
            }
            
            // Write proposed ADRs
            for adr in proposed {
                self.write_adr_table_row(&mut index_file, adr)?;
            }
            
            // Write amended ADRs
            for adr in amended {
                self.write_adr_table_row(&mut index_file, adr)?;
            }
            
            // Write superseded ADRs
            for adr in superseded {
                self.write_adr_table_row(&mut index_file, adr)?;
            }
            
            // Write deprecated ADRs
            for adr in deprecated {
                self.write_adr_table_row(&mut index_file, adr)?;
            }
            
            // Write rejected ADRs
            for adr in rejected {
                self.write_adr_table_row(&mut index_file, adr)?;
            }
            
            writeln!(index_file, "        </tbody>")?;
            writeln!(index_file, "    </table>")?;
            
            // Add graph if enabled
            if self.config.generate_graph {
                writeln!(index_file, "    <h2>ADR Relationship Graph</h2>")?;
                writeln!(index_file, "    <p><a href=\"adr-graph.html\">View ADR Relationship Graph</a></p>")?;
            }
        }
        
        writeln!(index_file, "    <p><a href=\"../index.html\">Back to documentation</a></p>")?;
        writeln!(index_file, "</body>")?;
        writeln!(index_file, "</html>")?;
        
        Ok(())
    }
    
    /// Write an ADR table row
    fn write_adr_table_row(&self, file: &mut File, adr: &ArchitectureDecisionRecord) -> io::Result<()> {
        writeln!(file, "        <tr>")?;
        writeln!(file, "            <td>{}</td>", adr.id)?;
        writeln!(file, "            <td><a href=\"{}.html\">{}</a></td>", adr.id, adr.title)?;
        
        // Status badge
        let badge_class = match adr.status {
            AdrStatus::Proposed => "badge-proposed",
            AdrStatus::Accepted => "badge-accepted",
            AdrStatus::Rejected => "badge-rejected",
            AdrStatus::Deprecated => "badge-deprecated",
            AdrStatus::Superseded => "badge-superseded",
            AdrStatus::Amended => "badge-amended",
        };
        
        writeln!(file, "            <td><span class=\"badge {}\">{}</span></td>", badge_class, adr.status)?;
        writeln!(file, "            <td>{}</td>", adr.date)?;
        
        // Tags
        writeln!(file, "            <td>")?;
        for tag in &adr.tags {
            writeln!(file, "                <span class=\"tag\">{}</span>", tag)?;
        }
        writeln!(file, "            </td>")?;
        
        writeln!(file, "        </tr>")?;
        
        Ok(())
    }
    
    /// Generate ADR page
    fn generate_adr_page(&self, output_dir: &Path, adr: &ArchitectureDecisionRecord) -> io::Result<()> {
        let page_path = output_dir.join(format!("{}.html", adr.id));
        let mut page_file = File::create(page_path)?;
        
        writeln!(page_file, "<!DOCTYPE html>")?;
        writeln!(page_file, "<html>")?;
        writeln!(page_file, "<head>")?;
        writeln!(page_file, "    <title>{} {}</title>", adr.id, adr.title)?;
        writeln!(page_file, "    <meta charset=\"UTF-8\">")?;
        writeln!(page_file, "    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">")?;
        writeln!(page_file, "    <style>")?;
        writeln!(page_file, "        body {{ font-family: Arial, sans-serif; margin: 0; padding: 20px; }}")?;
        writeln!(page_file, "        h1, h2, h3 {{ color: #333; }}")?;
        writeln!(page_file, "        .adr-meta {{ color: #666; font-size: 0.9em; margin-bottom: 20px; }}")?;
        writeln!(page_file, "        .badge {{ display: inline-block; padding: 3px 8px; border-radius: 3px; font-size: 0.8em; color: white; }}")?;
        writeln!(page_file, "        .badge-proposed {{ background-color: #6c757d; }}")?;
        writeln!(page_file, "        .badge-accepted {{ background-color: #28a745; }}")?;
        writeln!(page_file, "        .badge-rejected {{ background-color: #dc3545; }}")?;
        writeln!(page_file, "        .badge-deprecated {{ background-color: #6c757d; }}")?;
        writeln!(page_file, "        .badge-superseded {{ background-color: #fd7e14; }}")?;
        writeln!(page_file, "        .badge-amended {{ background-color: #17a2b8; }}")?;
        writeln!(page_file, "        .tag {{ display: inline-block; background-color: #f0f0f0; padding: 2px 6px; margin-right: 5px; border-radius: 3px; font-size: 0.8em; }}")?;
        writeln!(page_file, "        .section {{ margin-bottom: 20px; }}")?;
        writeln!(page_file, "    </style>")?;
        writeln!(page_file, "</head>")?;
        writeln!(page_file, "<body>")?;
        writeln!(page_file, "    <h1>{} {}</h1>", adr.id, adr.title)?;
        
        // Status badge
        let badge_class = match adr.status {
            AdrStatus::Proposed => "badge-proposed",
            AdrStatus::Accepted => "badge-accepted",
            AdrStatus::Rejected => "badge-rejected",
            AdrStatus::Deprecated => "badge-deprecated",
            AdrStatus::Superseded => "badge-superseded",
            AdrStatus::Amended => "badge-amended",
        };
        
        writeln!(page_file, "    <div class=\"adr-meta\">")?;
        writeln!(page_file, "        <div><strong>Status:</strong> <span class=\"badge {}\">{}</span></div>", badge_class, adr.status)?;
        writeln!(page_file, "        <div><strong>Date:</strong> {}</div>", adr.date)?;
        
        // Authors and approvers
        if !adr.authors.is_empty() {
            writeln!(page_file, "        <div><strong>Authors:</strong> {}</div>", adr.authors.join(", "))?;
        }
        
        if !adr.approvers.is_empty() {
            writeln!(page_file, "        <div><strong>Approvers:</strong> {}</div>", adr.approvers.join(", "))?;
        }
        
        // Tags
        if !adr.tags.is_empty() {
            writeln!(page_file, "        <div><strong>Tags:</strong> ")?;
            for tag in &adr.tags {
                writeln!(page_file, "            <span class=\"tag\">{}</span>", tag)?;
            }
            writeln!(page_file, "        </div>")?;
        }
        
        // Related ADRs
        if !adr.related_adrs.is_empty() {
            writeln!(page_file, "        <div><strong>Related ADRs:</strong> ")?;
            let mut first = true;
            for related_adr in &adr.related_adrs {
                if !first {
                    write!(page_file, ", ")?;
                }
                first = false;
                
                if self.adrs.contains_key(related_adr) {
                    write!(page_file, "<a href=\"{}.html\">{}</a>", related_adr, related_adr)?;
                } else {
                    write!(page_file, "{}", related_adr)?;
                }
            }
            writeln!(page_file, "</div>")?;
        }
        
        // Supersedes/Amends relationships
        if !adr.supersedes.is_empty() {
            writeln!(page_file, "        <div><strong>Supersedes:</strong> ")?;
            let mut first = true;
            for superseded_adr in &adr.supersedes {
                if !first {
                    write!(page_file, ", ")?;
                }
                first = false;
                
                if self.adrs.contains_key(superseded_adr) {
                    write!(page_file, "<a href=\"{}.html\">{}</a>", superseded_adr, superseded_adr)?;
                } else {
                    write!(page_file, "{}", superseded_adr)?;
                }
            }
            writeln!(page_file, "</div>")?;
        }
        
        if !adr.amends.is_empty() {
            writeln!(page_file, "        <div><strong>Amends:</strong> ")?;
            let mut first = true;
            for amended_adr in &adr.amends {
                if !first {
                    write!(page_file, ", ")?;
                }
                first = false;
                
                if self.adrs.contains_key(amended_adr) {
                    write!(page_file, "<a href=\"{}.html\">{}</a>", amended_adr, amended_adr)?;
                } else {
                    write!(page_file, "{}", amended_adr)?;
                }
            }
            writeln!(page_file, "</div>")?;
        }
        
        if let Some(superseded_by) = &adr.superseded_by {
            writeln!(page_file, "        <div><strong>Superseded by:</strong> ")?;
            if self.adrs.contains_key(superseded_by) {
                write!(page_file, "<a href=\"{}.html\">{}</a>", superseded_by, superseded_by)?;
            } else {
                write!(page_file, "{}", superseded_by)?;
            }
            writeln!(page_file, "</div>")?;
        }
        
        if !adr.amended_by.is_empty() {
            writeln!(page_file, "        <div><strong>Amended by:</strong> ")?;
            let mut first = true;
            for amending_adr in &adr.amended_by {
                if !first {
                    write!(page_file, ", ")?;
                }
                first = false;
                
                if self.adrs.contains_key(amending_adr) {
                    write!(page_file, "<a href=\"{}.html\">{}</a>", amending_adr, amending_adr)?;
                } else {
                    write!(page_file, "{}", amending_adr)?;
                }
            }
            writeln!(page_file, "</div>")?;
        }
        
        writeln!(page_file, "    </div>")?;
        
        // Main content sections
        writeln!(page_file, "    <div class=\"section\">")?;
        writeln!(page_file, "        <h2>Context</h2>")?;
        writeln!(page_file, "        <p>{}</p>", adr.context.replace("\n", "<br>"))?;
        writeln!(page_file, "    </div>")?;
        
        writeln!(page_file, "    <div class=\"section\">")?;
        writeln!(page_file, "        <h2>Decision</h2>")?;
        writeln!(page_file, "        <p>{}</p>", adr.decision.replace("\n", "<br>"))?;
        writeln!(page_file, "    </div>")?;
        
        writeln!(page_file, "    <div class=\"section\">")?;
        writeln!(page_file, "        <h2>Consequences</h2>")?;
        writeln!(page_file, "        <p>{}</p>", adr.consequences.replace("\n", "<br>"))?;
        writeln!(page_file, "    </div>")?;
        
        // Alternatives
        if !adr.alternatives.is_empty() {
            writeln!(page_file, "    <div class=\"section\">")?;
            writeln!(page_file, "        <h2>Alternatives Considered</h2>")?;
            
            for (i, alternative) in adr.alternatives.iter().enumerate() {
                writeln!(page_file, "        <h3>Alternative {}</h3>", i + 1)?;
                writeln!(page_file, "        <p>{}</p>", alternative.replace("\n", "<br>"))?;
            }
            
            writeln!(page_file, "    </div>")?;
        }
        
        // References
        if !adr.references.is_empty() {
            writeln!(page_file, "    <div class=\"section\">")?;
            writeln!(page_file, "        <h2>References</h2>")?;
            writeln!(page_file, "        <ul>")?;
            
            for reference in &adr.references {
                if reference.starts_with("http") {
                    writeln!(page_file, "            <li><a href=\"{}\">{}</a></li>", reference, reference)?;
                } else {
                    writeln!(page_file, "            <li>{}</li>", reference)?;
                }
            }
            
            writeln!(page_file, "        </ul>")?;
            writeln!(page_file, "    </div>")?;
        }
        
        writeln!(page_file, "    <p><a href=\"index.html\">Back to ADR index</a></p>")?;
        writeln!(page_file, "</body>")?;
        writeln!(page_file, "</html>")?;
        
        Ok(())
    }
    
    /// Generate ADR relationship graph
    fn generate_adr_graph(&self, output_dir: &Path) -> io::Result<()> {
        let graph_path = output_dir.join("adr-graph.html");
        let mut graph_file = File::create(graph_path)?;
        
        writeln!(graph_file, "<!DOCTYPE html>")?;
        writeln!(graph_file, "<html>")?;
        writeln!(graph_file, "<head>")?;
        writeln!(graph_file, "    <title>ADR Relationship Graph</title>")?;
        writeln!(graph_file, "    <meta charset=\"UTF-8\">")?;
        writeln!(graph_file, "    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">")?;
        writeln!(graph_file, "    <script src=\"https://cdnjs.cloudflare.com/ajax/libs/mermaid/8.13.10/mermaid.min.js\"></script>")?;
        writeln!(graph_file, "    <style>")?;
        writeln!(graph_file, "        body {{ font-family: Arial, sans-serif; margin: 0; padding: 20px; }}")?;
        writeln!(graph_file, "        h1, h2 {{ color: #333; }}")?;
        writeln!(graph_file, "        .graph-container {{ margin-top: 20px; }}")?;
        writeln!(graph_file, "    </style>")?;
        writeln!(graph_file, "</head>")?;
        writeln!(graph_file, "<body>")?;
        writeln!(graph_file, "    <h1>ADR Relationship Graph</h1>")?;
        
        writeln!(graph_file, "    <div class=\"graph-container\">")?;
        writeln!(graph_file, "        <pre class=\"mermaid\">")?;
        writeln!(graph_file, "graph TD")?;
        
        // Define nodes
        for adr in self.adrs.values() {
            let node_style = match adr.status {
                AdrStatus::Proposed => "style {} fill:#f9f9f9,stroke:#6c757d",
                AdrStatus::Accepted => "style {} fill:#e6ffe6,stroke:#28a745",
                AdrStatus::Rejected => "style {} fill:#ffe6e6,stroke:#dc3545",
                AdrStatus::Deprecated => "style {} fill:#f9f9f9,stroke:#6c757d",
                AdrStatus::Superseded => "style {} fill:#fff3e6,stroke:#fd7e14",
                AdrStatus::Amended => "style {} fill:#e6f9ff,stroke:#17a2b8",
            };
            
            writeln!(graph_file, "    {}[\"{} - {}\"]", adr.id, adr.id, adr.title)?;
            writeln!(graph_file, "    {}", node_style.replace("{}", &adr.id))?;
        }
        
        // Define relationships
        for adr in self.adrs.values() {
            // Supersedes relationships
            for superseded_adr in &adr.supersedes {
                if self.adrs.contains_key(superseded_adr) {
                    writeln!(graph_file, "    {} -->|supersedes| {}", adr.id, superseded_adr)?;
                }
            }
            
            // Amends relationships
            for amended_adr in &adr.amends {
                if self.adrs.contains_key(amended_adr) {
                    writeln!(graph_file, "    {} -->|amends| {}", adr.id, amended_adr)?;
                }
            }
            
            // Related ADRs (if not already connected by supersedes/amends)
            for related_adr in &adr.related_adrs {
                if self.adrs.contains_key(related_adr) {
                    // Check if already connected
                    let already_connected = 
                        adr.supersedes.contains(related_adr) ||
                        adr.amends.contains(related_adr) ||
                        (adr.superseded_by.as_ref().map_or(false, |id| id == related_adr)) ||
                        adr.amended_by.contains(related_adr);
                    
                    if !already_connected {
                        writeln!(graph_file, "    {} ---|related| {}", adr.id, related_adr)?;
                    }
                }
            }
        }
        
        writeln!(graph_file, "        </pre>")?;
        writeln!(graph_file, "    </div>")?;
        
        writeln!(graph_file, "    <script>")?;
        writeln!(graph_file, "        mermaid.initialize({{ startOnLoad: true }});")?;
        writeln!(graph_file, "    </script>")?;
        
        writeln!(graph_file, "    <p><a href=\"index.html\">Back to ADR index</a></p>")?;
        writeln!(graph_file, "</body>")?;
        writeln!(graph_file, "</html>")?;
        
        Ok(())
    }
    
    /// Create example ADRs
    pub fn create_example_adrs(&mut self) -> io::Result<()> {
        info!("Creating example ADRs");
        
        // ADR 1: Use Rust for Backend
        let adr1 = ArchitectureDecisionRecord::new(
            "ADR-001",
            "Use Rust for Backend Development",
            AdrStatus::Accepted,
            "2023-01-15",
            "We need to choose a programming language for the backend development of our application. The language should be performant, safe, and have good support for concurrent programming.",
            "We will use Rust for backend development.",
            "Using Rust will provide memory safety without garbage collection, excellent performance, and strong concurrency support. The learning curve may be steeper than some alternatives, but the long-term benefits outweigh this cost."
        )
        .with_author("Jane Developer")
        .with_approver("John Architect")
        .with_tag("backend")
        .with_tag("language")
        .with_alternative("Go: Offers good performance and concurrency, but lacks Rust's memory safety guarantees without garbage collection.")
        .with_alternative("Node.js: Familiar to many developers, but may have performance limitations for our use case.")
        .with_reference("https://www.rust-lang.org/")
        .with_reference("Performance benchmarks: https://benchmarksgame-team.pages.debian.net/benchmarksgame/fastest/rust.html");
        
        // ADR 2: Adopt Actor Model
        let adr2 = ArchitectureDecisionRecord::new(
            "ADR-002",
            "Adopt Actor Model for Concurrency",
            AdrStatus::Accepted,
            "2023-02-10",
            "We need a concurrency model that allows for safe and efficient parallel processing in our backend system. The model should be scalable and help prevent common concurrency issues like race conditions and deadlocks.",
            "We will adopt the Actor Model for concurrency, using the Kameo actor framework for Rust.",
            "The Actor Model provides a high-level abstraction for concurrent programming that helps avoid many common pitfalls. Actors communicate through message passing, which eliminates shared mutable state. This approach aligns well with Rust's ownership model and will help us build a robust, concurrent system."
        )
        .with_author("Jane Developer")
        .with_approver("John Architect")
        .with_tag("concurrency")
        .with_tag("architecture")
        .with_related_adr("ADR-001")
        .with_alternative("Traditional multithreading with locks: More error-prone and difficult to reason about.")
        .with_alternative("Async/await with futures: Good for I/O-bound tasks but doesn't solve all concurrency challenges.")
        .with_reference("https://en.wikipedia.org/wiki/Actor_model")
        .with_reference("Kameo actor framework documentation");
        
        // ADR 3: Local-First Architecture
        let adr3 = ArchitectureDecisionRecord::new(
            "ADR-003",
            "Adopt Local-First Architecture",
            AdrStatus::Accepted,
            "2023-03-05",
            "We need to decide on the overall architecture approach for our application. Users need to be able to work offline and have their data synchronized when they reconnect.",
            "We will adopt a local-first architecture where data is primarily stored locally and synchronized with the server when possible.",
            "A local-first approach will allow users to work offline, provide better performance, and give users ownership of their data. It will require more complex synchronization logic, but the benefits for our use case outweigh this cost."
        )
        .with_author("John Architect")
        .with_approver("Sarah CTO")
        .with_tag("architecture")
        .with_tag("offline")
        .with_related_adr("ADR-001")
        .with_related_adr("ADR-002")
        .with_alternative("Traditional client-server: Simpler but requires constant connectivity.")
        .with_alternative("Progressive Web App with service workers: Good for web but doesn't provide the same level of offline capability for our desktop application.")
        .with_reference("Local-First Software: https://www.inkandswitch.com/local-first/");
        
        // ADR 4: SQLite for Local Storage
        let adr4 = ArchitectureDecisionRecord::new(
            "ADR-004",
            "Use SQLite for Local Storage",
            AdrStatus::Accepted,
            "2023-03-20",
            "We need a local storage solution for our local-first architecture. The solution should be reliable, performant, and well-supported in our technology stack.",
            "We will use SQLite for local storage in our application.",
            "SQLite provides a reliable, self-contained database that works well for local storage. It has excellent performance, a small footprint, and good support in Rust through the SQLx library. It also supports the SQL standard, making it familiar to many developers."
        )
        .with_author("Jane Developer")
        .with_approver("John Architect")
        .with_tag("storage")
        .with_tag("database")
        .with_related_adr("ADR-003")
        .with_alternative("IndexedDB: Good for web applications but not as well-suited for our desktop application.")
        .with_alternative("Custom file format: Would require more development effort and lack the maturity of SQLite.")
        .with_reference("https://www.sqlite.org/")
        .with_reference("SQLx documentation: https://github.com/launchbadge/sqlx");
        
        // ADR 5: Superseded ADR
        let adr5 = ArchitectureDecisionRecord::new(
            "ADR-005",
            "Use REST for API Design",
            AdrStatus::Superseded,
            "2023-04-10",
            "We need to decide on an API design approach for communication between our frontend and backend components.",
            "We will use REST for our API design, following RESTful principles for resource naming and HTTP method usage.",
            "REST is a well-established pattern that is familiar to most developers. It works well with HTTP and provides a clear structure for our API. The stateless nature of REST also aligns with our distributed architecture."
        )
        .with_author("Jane Developer")
        .with_approver("John Architect")
        .with_tag("api")
        .with_tag("communication")
        .with_related_adr("ADR-001")
        .with_alternative("GraphQL: More flexible but adds complexity.")
        .with_alternative("gRPC: Better performance but less familiar and harder to debug.")
        .with_reference("RESTful API design: https://restfulapi.net/")
        .superseded_by("ADR-006");
        
        // ADR 6: Supersedes ADR-005
        let adr6 = ArchitectureDecisionRecord::new(
            "ADR-006",
            "Use GraphQL for API Design",
            AdrStatus::Accepted,
            "2023-05-15",
            "After implementing our initial REST API, we've found that clients often need to request data in varying shapes, leading to either over-fetching or under-fetching of data. We need a more flexible API design.",
            "We will migrate from REST to GraphQL for our API design.",
            "GraphQL will provide more flexibility for clients to request exactly the data they need. This will reduce network traffic and improve performance. It also provides a strongly-typed schema, which aligns with our focus on type safety."
        )
        .with_author("Jane Developer")
        .with_approver("John Architect")
        .with_tag("api")
        .with_tag("communication")
        .with_related_adr("ADR-001")
        .with_alternative("Expand REST API with more endpoints: Would lead to API bloat and maintenance challenges.")
        .with_alternative("Hybrid approach with both REST and GraphQL: Adds complexity without clear benefits.")
        .with_reference("GraphQL: https://graphql.org/")
        .supersedes("ADR-005");
        
        // Add ADRs to the collection
        self.create_adr(adr1)?;
        self.create_adr(adr2)?;
        self.create_adr(adr3)?;
        self.create_adr(adr4)?;
        self.create_adr(adr5)?;
        self.create_adr(adr6)?;
        
        info!("Created {} example ADRs", self.adrs.len());
        Ok(())
    }
}

/// Global ADR manager
lazy_static::lazy_static! {
    static ref ADR_MANAGER: Arc<Mutex<AdrManager>> = Arc::new(Mutex::new(
        AdrManager::new(AdrConfig::default(), DocsGenConfig::default())
    ));
}

/// Get the global ADR manager
pub fn get_adr_manager() -> Arc<Mutex<AdrManager>> {
    ADR_MANAGER.clone()
}

/// Configure ADR generation
pub fn configure(config: AdrConfig, base_config: DocsGenConfig) {
    let mut manager = ADR_MANAGER.lock().unwrap();
    *manager = AdrManager::new(config, base_config);
}

/// Generate ADR documentation
pub fn generate_adr_docs(output_dir: &Path) -> io::Result<()> {
    let mut manager = ADR_MANAGER.lock().unwrap();
    
    // Check if ADR documentation generation is enabled
    if !manager.config.enabled {
        info!("ADR documentation generation is disabled");
        return Ok(());
    }
    
    // Create example ADRs if none exist
    if manager.adrs.is_empty() {
        manager.create_example_adrs()?;
    }
    
    // Generate ADR documentation
    manager.generate_adr_docs(output_dir)?;
    
    info!("ADR documentation generated successfully");
    Ok(())
}

/// Initialize the ADR system
pub fn init() {
    info!("Initializing ADR system");
}