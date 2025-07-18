//! Visual Documentation for Complex Systems
//!
//! This module provides tools for generating visual documentation for complex
//! systems, including architecture diagrams, component relationships, data flow
//! diagrams, and state machines.

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use serde::{Serialize, Deserialize};
use tracing::{debug, info, warn, error};

use super::DocsGenConfig;

/// Visual documentation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualDocsConfig {
    /// Whether visual documentation is enabled
    pub enabled: bool,
    /// Whether to generate architecture diagrams
    pub generate_architecture_diagrams: bool,
    /// Whether to generate component diagrams
    pub generate_component_diagrams: bool,
    /// Whether to generate data flow diagrams
    pub generate_data_flow_diagrams: bool,
    /// Whether to generate state machine diagrams
    pub generate_state_machine_diagrams: bool,
    /// Whether to generate dependency graphs
    pub generate_dependency_graphs: bool,
    /// Whether to generate interactive diagrams
    pub generate_interactive_diagrams: bool,
    /// Additional options
    pub options: HashMap<String, String>,
}

impl Default for VisualDocsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            generate_architecture_diagrams: true,
            generate_component_diagrams: true,
            generate_data_flow_diagrams: true,
            generate_state_machine_diagrams: true,
            generate_dependency_graphs: true,
            generate_interactive_diagrams: true,
            options: HashMap::new(),
        }
    }
}

/// Diagram type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiagramType {
    /// Architecture diagram
    Architecture,
    /// Component diagram
    Component,
    /// Data flow diagram
    DataFlow,
    /// State machine diagram
    StateMachine,
    /// Dependency graph
    DependencyGraph,
    /// Custom diagram
    Custom(String),
}

/// Diagram format
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiagramFormat {
    /// Mermaid.js format
    Mermaid,
    /// PlantUML format
    PlantUML,
    /// Graphviz DOT format
    Graphviz,
    /// SVG format
    SVG,
    /// Custom format
    Custom(String),
}

/// Visual diagram
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagram {
    /// Diagram ID
    pub id: String,
    /// Diagram title
    pub title: String,
    /// Diagram description
    pub description: String,
    /// Diagram type
    pub diagram_type: DiagramType,
    /// Diagram format
    pub format: DiagramFormat,
    /// Diagram content
    pub content: String,
    /// Diagram tags
    pub tags: Vec<String>,
    /// Diagram author
    pub author: Option<String>,
    /// Diagram creation date
    pub created_at: String,
    /// Diagram last updated date
    pub updated_at: String,
    /// Related components
    pub related_components: Vec<String>,
    /// Diagram version
    pub version: String,
}

impl Diagram {
    /// Create a new diagram
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        description: impl Into<String>,
        diagram_type: DiagramType,
        format: DiagramFormat,
        content: impl Into<String>,
    ) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        
        Self {
            id: id.into(),
            title: title.into(),
            description: description.into(),
            diagram_type,
            format,
            content: content.into(),
            tags: Vec::new(),
            author: None,
            created_at: now.clone(),
            updated_at: now,
            related_components: Vec::new(),
            version: "1.0.0".to_string(),
        }
    }
    
    /// Add a tag to the diagram
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }
    
    /// Set the diagram author
    pub fn with_author(mut self, author: impl Into<String>) -> Self {
        self.author = Some(author.into());
        self
    }
    
    /// Add a related component to the diagram
    pub fn with_related_component(mut self, component: impl Into<String>) -> Self {
        self.related_components.push(component.into());
        self
    }
    
    /// Set the diagram version
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }
}

/// Component relationship type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationshipType {
    /// Dependency relationship
    Dependency,
    /// Association relationship
    Association,
    /// Aggregation relationship
    Aggregation,
    /// Composition relationship
    Composition,
    /// Inheritance relationship
    Inheritance,
    /// Implementation relationship
    Implementation,
    /// Custom relationship
    Custom(String),
}

/// Component relationship
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentRelationship {
    /// Source component
    pub source: String,
    /// Target component
    pub target: String,
    /// Relationship type
    pub relationship_type: RelationshipType,
    /// Relationship label
    pub label: Option<String>,
    /// Relationship description
    pub description: Option<String>,
}

impl ComponentRelationship {
    /// Create a new component relationship
    pub fn new(
        source: impl Into<String>,
        target: impl Into<String>,
        relationship_type: RelationshipType,
    ) -> Self {
        Self {
            source: source.into(),
            target: target.into(),
            relationship_type,
            label: None,
            description: None,
        }
    }
    
    /// Set the relationship label
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
    
    /// Set the relationship description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}

/// System component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Component {
    /// Component ID
    pub id: String,
    /// Component name
    pub name: String,
    /// Component description
    pub description: String,
    /// Component type
    pub component_type: String,
    /// Component responsibilities
    pub responsibilities: Vec<String>,
    /// Component dependencies
    pub dependencies: Vec<String>,
    /// Component interfaces
    pub interfaces: Vec<String>,
    /// Component tags
    pub tags: Vec<String>,
    /// Component author
    pub author: Option<String>,
    /// Component creation date
    pub created_at: String,
    /// Component last updated date
    pub updated_at: String,
}

impl Component {
    /// Create a new component
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        component_type: impl Into<String>,
    ) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        
        Self {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            component_type: component_type.into(),
            responsibilities: Vec::new(),
            dependencies: Vec::new(),
            interfaces: Vec::new(),
            tags: Vec::new(),
            author: None,
            created_at: now.clone(),
            updated_at: now,
        }
    }
    
    /// Add a responsibility to the component
    pub fn with_responsibility(mut self, responsibility: impl Into<String>) -> Self {
        self.responsibilities.push(responsibility.into());
        self
    }
    
    /// Add a dependency to the component
    pub fn with_dependency(mut self, dependency: impl Into<String>) -> Self {
        self.dependencies.push(dependency.into());
        self
    }
    
    /// Add an interface to the component
    pub fn with_interface(mut self, interface: impl Into<String>) -> Self {
        self.interfaces.push(interface.into());
        self
    }
    
    /// Add a tag to the component
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }
    
    /// Set the component author
    pub fn with_author(mut self, author: impl Into<String>) -> Self {
        self.author = Some(author.into());
        self
    }
}

/// System architecture
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemArchitecture {
    /// Architecture ID
    pub id: String,
    /// Architecture name
    pub name: String,
    /// Architecture description
    pub description: String,
    /// Architecture components
    pub components: HashMap<String, Component>,
    /// Component relationships
    pub relationships: Vec<ComponentRelationship>,
    /// Architecture diagrams
    pub diagrams: HashMap<String, Diagram>,
    /// Architecture version
    pub version: String,
    /// Architecture author
    pub author: Option<String>,
    /// Architecture creation date
    pub created_at: String,
    /// Architecture last updated date
    pub updated_at: String,
}

impl SystemArchitecture {
    /// Create a new system architecture
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        
        Self {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            components: HashMap::new(),
            relationships: Vec::new(),
            diagrams: HashMap::new(),
            version: "1.0.0".to_string(),
            author: None,
            created_at: now.clone(),
            updated_at: now,
        }
    }
    
    /// Add a component to the architecture
    pub fn add_component(&mut self, component: Component) {
        self.components.insert(component.id.clone(), component);
    }
    
    /// Add a relationship to the architecture
    pub fn add_relationship(&mut self, relationship: ComponentRelationship) {
        self.relationships.push(relationship);
    }
    
    /// Add a diagram to the architecture
    pub fn add_diagram(&mut self, diagram: Diagram) {
        self.diagrams.insert(diagram.id.clone(), diagram);
    }
    
    /// Set the architecture version
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }
    
    /// Set the architecture author
    pub fn with_author(mut self, author: impl Into<String>) -> Self {
        self.author = Some(author.into());
        self
    }
}

/// Visual documentation generator
#[derive(Debug)]
pub struct VisualDocsGenerator {
    /// Configuration
    pub config: VisualDocsConfig,
    /// Base documentation generator configuration
    pub base_config: DocsGenConfig,
    /// System architectures
    pub architectures: HashMap<String, SystemArchitecture>,
}

impl VisualDocsGenerator {
    /// Create a new visual documentation generator
    pub fn new(config: VisualDocsConfig, base_config: DocsGenConfig) -> Self {
        Self {
            config,
            base_config,
            architectures: HashMap::new(),
        }
    }
    
    /// Load architectures from a directory
    pub fn load_architectures(&mut self, dir: &Path) -> io::Result<()> {
        info!("Loading system architectures from {}", dir.display());
        
        let architectures_dir = dir.join("architectures");
        if architectures_dir.exists() && architectures_dir.is_dir() {
            for entry in fs::read_dir(architectures_dir)? {
                let entry = entry?;
                let path = entry.path();
                
                if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                    let mut file = File::open(&path)?;
                    let mut contents = String::new();
                    file.read_to_string(&mut contents)?;
                    
                    match serde_json::from_str::<SystemArchitecture>(&contents) {
                        Ok(architecture) => {
                            self.architectures.insert(architecture.id.clone(), architecture);
                        }
                        Err(err) => {
                            warn!("Failed to parse architecture file {}: {}", path.display(), err);
                        }
                    }
                }
            }
        }
        
        info!("Loaded {} system architectures", self.architectures.len());
        Ok(())
    }
    
    /// Generate visual documentation
    pub fn generate_visual_docs(&self, output_dir: &Path) -> io::Result<()> {
        if !self.config.enabled {
            info!("Visual documentation is disabled");
            return Ok(());
        }
        
        info!("Generating visual documentation");
        
        // Create output directory
        let visual_dir = output_dir.join("visual");
        fs::create_dir_all(&visual_dir)?;
        
        // Generate index page
        self.generate_index(&visual_dir)?;
        
        // Generate architecture pages
        for architecture in self.architectures.values() {
            self.generate_architecture_page(&visual_dir, architecture)?;
            
            // Generate component pages
            for component in architecture.components.values() {
                self.generate_component_page(&visual_dir, architecture, component)?;
            }
            
            // Generate diagram pages
            for diagram in architecture.diagrams.values() {
                match diagram.diagram_type {
                    DiagramType::Architecture => {
                        if self.config.generate_architecture_diagrams {
                            self.generate_diagram_page(&visual_dir, architecture, diagram)?;
                        }
                    }
                    DiagramType::Component => {
                        if self.config.generate_component_diagrams {
                            self.generate_diagram_page(&visual_dir, architecture, diagram)?;
                        }
                    }
                    DiagramType::DataFlow => {
                        if self.config.generate_data_flow_diagrams {
                            self.generate_diagram_page(&visual_dir, architecture, diagram)?;
                        }
                    }
                    DiagramType::StateMachine => {
                        if self.config.generate_state_machine_diagrams {
                            self.generate_diagram_page(&visual_dir, architecture, diagram)?;
                        }
                    }
                    DiagramType::DependencyGraph => {
                        if self.config.generate_dependency_graphs {
                            self.generate_diagram_page(&visual_dir, architecture, diagram)?;
                        }
                    }
                    _ => {
                        // Generate custom diagrams by default
                        self.generate_diagram_page(&visual_dir, architecture, diagram)?;
                    }
                }
            }
        }
        
        info!("Visual documentation generated successfully");
        Ok(())
    }
    
    /// Generate index page
    fn generate_index(&self, output_dir: &Path) -> io::Result<()> {
        let index_path = output_dir.join("index.html");
        let mut index_file = File::create(index_path)?;
        
        writeln!(index_file, "<!DOCTYPE html>")?;
        writeln!(index_file, "<html>")?;
        writeln!(index_file, "<head>")?;
        writeln!(index_file, "    <title>Visual Documentation</title>")?;
        writeln!(index_file, "    <meta charset=\"UTF-8\">")?;
        writeln!(index_file, "    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">")?;
        writeln!(index_file, "    <style>")?;
        writeln!(index_file, "        body {{ font-family: Arial, sans-serif; margin: 0; padding: 20px; }}")?;
        writeln!(index_file, "        h1, h2 {{ color: #333; }}")?;
        writeln!(index_file, "        .architecture {{ margin-bottom: 20px; padding: 15px; border: 1px solid #ddd; border-radius: 5px; }}")?;
        writeln!(index_file, "        .architecture h3 {{ margin-top: 0; }}")?;
        writeln!(index_file, "        .architecture-meta {{ color: #666; font-size: 0.9em; margin-bottom: 10px; }}")?;
        writeln!(index_file, "        .component-count {{ margin-top: 10px; color: #666; }}")?;
        writeln!(index_file, "    </style>")?;
        writeln!(index_file, "</head>")?;
        writeln!(index_file, "<body>")?;
        writeln!(index_file, "    <h1>Visual Documentation</h1>")?;
        
        if self.architectures.is_empty() {
            writeln!(index_file, "    <p>No system architectures available.</p>")?;
        } else {
            writeln!(index_file, "    <h2>System Architectures</h2>")?;
            
            for architecture in self.architectures.values() {
                writeln!(index_file, "    <div class=\"architecture\">")?;
                writeln!(index_file, "        <h3><a href=\"architecture_{}.html\">{}</a></h3>", architecture.id, architecture.name)?;
                writeln!(index_file, "        <div class=\"architecture-meta\">")?;
                writeln!(index_file, "            <span class=\"version\">Version: {}</span>", architecture.version)?;
                if let Some(author) = &architecture.author {
                    writeln!(index_file, "            <span class=\"author\">Author: {}</span>", author)?;
                }
                writeln!(index_file, "        </div>")?;
                writeln!(index_file, "        <p>{}</p>", architecture.description)?;
                writeln!(index_file, "        <div class=\"component-count\">")?;
                writeln!(index_file, "            <span>{} components</span>", architecture.components.len())?;
                writeln!(index_file, "            <span>{} relationships</span>", architecture.relationships.len())?;
                writeln!(index_file, "            <span>{} diagrams</span>", architecture.diagrams.len())?;
                writeln!(index_file, "        </div>")?;
                writeln!(index_file, "    </div>")?;
            }
        }
        
        writeln!(index_file, "    <p><a href=\"../index.html\">Back to documentation</a></p>")?;
        writeln!(index_file, "</body>")?;
        writeln!(index_file, "</html>")?;
        
        Ok(())
    }
    
    /// Generate architecture page
    fn generate_architecture_page(&self, output_dir: &Path, architecture: &SystemArchitecture) -> io::Result<()> {
        let page_path = output_dir.join(format!("architecture_{}.html", architecture.id));
        let mut page_file = File::create(page_path)?;
        
        writeln!(page_file, "<!DOCTYPE html>")?;
        writeln!(page_file, "<html>")?;
        writeln!(page_file, "<head>")?;
        writeln!(page_file, "    <title>Architecture: {}</title>", architecture.name)?;
        writeln!(page_file, "    <meta charset=\"UTF-8\">")?;
        writeln!(page_file, "    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">")?;
        writeln!(page_file, "    <style>")?;
        writeln!(page_file, "        body {{ font-family: Arial, sans-serif; margin: 0; padding: 20px; }}")?;
        writeln!(page_file, "        h1, h2, h3 {{ color: #333; }}")?;
        writeln!(page_file, "        .architecture-meta {{ color: #666; font-size: 0.9em; margin-bottom: 20px; }}")?;
        writeln!(page_file, "        .component {{ margin-bottom: 10px; }}")?;
        writeln!(page_file, "        .diagram {{ margin-bottom: 10px; }}")?;
        writeln!(page_file, "        .relationship {{ margin-bottom: 5px; }}")?;
        writeln!(page_file, "    </style>")?;
        writeln!(page_file, "</head>")?;
        writeln!(page_file, "<body>")?;
        writeln!(page_file, "    <h1>Architecture: {}</h1>", architecture.name)?;
        
        writeln!(page_file, "    <div class=\"architecture-meta\">")?;
        writeln!(page_file, "        <div>Version: {}</div>", architecture.version)?;
        if let Some(author) = &architecture.author {
            writeln!(page_file, "        <div>Author: {}</div>", author)?;
        }
        writeln!(page_file, "        <div>Created: {}</div>", architecture.created_at)?;
        writeln!(page_file, "        <div>Updated: {}</div>", architecture.updated_at)?;
        writeln!(page_file, "    </div>")?;
        
        writeln!(page_file, "    <p>{}</p>", architecture.description)?;
        
        // Write diagrams section
        if !architecture.diagrams.is_empty() {
            writeln!(page_file, "    <h2>Diagrams</h2>")?;
            
            // Group diagrams by type
            let mut architecture_diagrams = Vec::new();
            let mut component_diagrams = Vec::new();
            let mut data_flow_diagrams = Vec::new();
            let mut state_machine_diagrams = Vec::new();
            let mut dependency_diagrams = Vec::new();
            let mut other_diagrams = Vec::new();
            
            for diagram in architecture.diagrams.values() {
                match diagram.diagram_type {
                    DiagramType::Architecture => architecture_diagrams.push(diagram),
                    DiagramType::Component => component_diagrams.push(diagram),
                    DiagramType::DataFlow => data_flow_diagrams.push(diagram),
                    DiagramType::StateMachine => state_machine_diagrams.push(diagram),
                    DiagramType::DependencyGraph => dependency_diagrams.push(diagram),
                    _ => other_diagrams.push(diagram),
                }
            }
            
            // Write architecture diagrams
            if !architecture_diagrams.is_empty() && self.config.generate_architecture_diagrams {
                writeln!(page_file, "    <h3>Architecture Diagrams</h3>")?;
                for diagram in architecture_diagrams {
                    writeln!(page_file, "    <div class=\"diagram\">")?;
                    writeln!(page_file, "        <a href=\"diagram_{}_{}.html\">{}</a>", architecture.id, diagram.id, diagram.title)?;
                    writeln!(page_file, "    </div>")?;
                }
            }
            
            // Write component diagrams
            if !component_diagrams.is_empty() && self.config.generate_component_diagrams {
                writeln!(page_file, "    <h3>Component Diagrams</h3>")?;
                for diagram in component_diagrams {
                    writeln!(page_file, "    <div class=\"diagram\">")?;
                    writeln!(page_file, "        <a href=\"diagram_{}_{}.html\">{}</a>", architecture.id, diagram.id, diagram.title)?;
                    writeln!(page_file, "    </div>")?;
                }
            }
            
            // Write data flow diagrams
            if !data_flow_diagrams.is_empty() && self.config.generate_data_flow_diagrams {
                writeln!(page_file, "    <h3>Data Flow Diagrams</h3>")?;
                for diagram in data_flow_diagrams {
                    writeln!(page_file, "    <div class=\"diagram\">")?;
                    writeln!(page_file, "        <a href=\"diagram_{}_{}.html\">{}</a>", architecture.id, diagram.id, diagram.title)?;
                    writeln!(page_file, "    </div>")?;
                }
            }
            
            // Write state machine diagrams
            if !state_machine_diagrams.is_empty() && self.config.generate_state_machine_diagrams {
                writeln!(page_file, "    <h3>State Machine Diagrams</h3>")?;
                for diagram in state_machine_diagrams {
                    writeln!(page_file, "    <div class=\"diagram\">")?;
                    writeln!(page_file, "        <a href=\"diagram_{}_{}.html\">{}</a>", architecture.id, diagram.id, diagram.title)?;
                    writeln!(page_file, "    </div>")?;
                }
            }
            
            // Write dependency diagrams
            if !dependency_diagrams.is_empty() && self.config.generate_dependency_graphs {
                writeln!(page_file, "    <h3>Dependency Graphs</h3>")?;
                for diagram in dependency_diagrams {
                    writeln!(page_file, "    <div class=\"diagram\">")?;
                    writeln!(page_file, "        <a href=\"diagram_{}_{}.html\">{}</a>", architecture.id, diagram.id, diagram.title)?;
                    writeln!(page_file, "    </div>")?;
                }
            }
            
            // Write other diagrams
            if !other_diagrams.is_empty() {
                writeln!(page_file, "    <h3>Other Diagrams</h3>")?;
                for diagram in other_diagrams {
                    writeln!(page_file, "    <div class=\"diagram\">")?;
                    writeln!(page_file, "        <a href=\"diagram_{}_{}.html\">{}</a>", architecture.id, diagram.id, diagram.title)?;
                    writeln!(page_file, "    </div>")?;
                }
            }
        }
        
        // Write components section
        if !architecture.components.is_empty() {
            writeln!(page_file, "    <h2>Components</h2>")?;
            
            // Group components by type
            let mut component_types = HashMap::new();
            
            for component in architecture.components.values() {
                let components = component_types.entry(component.component_type.clone()).or_insert_with(Vec::new);
                components.push(component);
            }
            
            // Write components by type
            for (component_type, components) in component_types {
                writeln!(page_file, "    <h3>{}</h3>", component_type)?;
                
                for component in components {
                    writeln!(page_file, "    <div class=\"component\">")?;
                    writeln!(page_file, "        <a href=\"component_{}_{}.html\">{}</a>", architecture.id, component.id, component.name)?;
                    writeln!(page_file, "    </div>")?;
                }
            }
        }
        
        // Write relationships section
        if !architecture.relationships.is_empty() {
            writeln!(page_file, "    <h2>Relationships</h2>")?;
            
            // Group relationships by type
            let mut relationship_types = HashMap::new();
            
            for relationship in &architecture.relationships {
                let relationship_type = match &relationship.relationship_type {
                    RelationshipType::Dependency => "Dependency",
                    RelationshipType::Association => "Association",
                    RelationshipType::Aggregation => "Aggregation",
                    RelationshipType::Composition => "Composition",
                    RelationshipType::Inheritance => "Inheritance",
                    RelationshipType::Implementation => "Implementation",
                    RelationshipType::Custom(name) => name,
                };
                
                let relationships = relationship_types.entry(relationship_type.to_string()).or_insert_with(Vec::new);
                relationships.push(relationship);
            }
            
            // Write relationships by type
            for (relationship_type, relationships) in relationship_types {
                writeln!(page_file, "    <h3>{}</h3>", relationship_type)?;
                
                for relationship in relationships {
                    let source_component = architecture.components.get(&relationship.source);
                    let target_component = architecture.components.get(&relationship.target);
                    
                    let source_name = source_component.map_or(&relationship.source, |c| &c.name);
                    let target_name = target_component.map_or(&relationship.target, |c| &c.name);
                    
                    writeln!(page_file, "    <div class=\"relationship\">")?;
                    
                    if let Some(source) = source_component {
                        writeln!(page_file, "        <a href=\"component_{}_{}.html\">{}</a>", architecture.id, source.id, source.name)?;
                    } else {
                        writeln!(page_file, "        {}", source_name)?;
                    }
                    
                    if let Some(label) = &relationship.label {
                        writeln!(page_file, "        <span> {} </span>", label)?;
                    } else {
                        writeln!(page_file, "        <span> → </span>")?;
                    }
                    
                    if let Some(target) = target_component {
                        writeln!(page_file, "        <a href=\"component_{}_{}.html\">{}</a>", architecture.id, target.id, target.name)?;
                    } else {
                        writeln!(page_file, "        {}", target_name)?;
                    }
                    
                    if let Some(description) = &relationship.description {
                        writeln!(page_file, "        <div>{}</div>", description)?;
                    }
                    
                    writeln!(page_file, "    </div>")?;
                }
            }
        }
        
        writeln!(page_file, "    <p><a href=\"index.html\">Back to index</a></p>")?;
        writeln!(page_file, "</body>")?;
        writeln!(page_file, "</html>")?;
        
        Ok(())
    }
    
    /// Generate component page
    fn generate_component_page(&self, output_dir: &Path, architecture: &SystemArchitecture, component: &Component) -> io::Result<()> {
        let page_path = output_dir.join(format!("component_{}_{}.html", architecture.id, component.id));
        let mut page_file = File::create(page_path)?;
        
        writeln!(page_file, "<!DOCTYPE html>")?;
        writeln!(page_file, "<html>")?;
        writeln!(page_file, "<head>")?;
        writeln!(page_file, "    <title>Component: {}</title>", component.name)?;
        writeln!(page_file, "    <meta charset=\"UTF-8\">")?;
        writeln!(page_file, "    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">")?;
        writeln!(page_file, "    <style>")?;
        writeln!(page_file, "        body {{ font-family: Arial, sans-serif; margin: 0; padding: 20px; }}")?;
        writeln!(page_file, "        h1, h2, h3 {{ color: #333; }}")?;
        writeln!(page_file, "        .component-meta {{ color: #666; font-size: 0.9em; margin-bottom: 20px; }}")?;
        writeln!(page_file, "        .component-tags {{ margin-top: 10px; }}")?;
        writeln!(page_file, "        .tag {{ display: inline-block; background-color: #f0f0f0; padding: 3px 8px; margin-right: 5px; border-radius: 3px; font-size: 0.8em; }}")?;
        writeln!(page_file, "        .relationship {{ margin-bottom: 5px; }}")?;
        writeln!(page_file, "    </style>")?;
        writeln!(page_file, "</head>")?;
        writeln!(page_file, "<body>")?;
        writeln!(page_file, "    <h1>Component: {}</h1>", component.name)?;
        
        writeln!(page_file, "    <div class=\"component-meta\">")?;
        writeln!(page_file, "        <div>Type: {}</div>", component.component_type)?;
        if let Some(author) = &component.author {
            writeln!(page_file, "        <div>Author: {}</div>", author)?;
        }
        writeln!(page_file, "        <div>Created: {}</div>", component.created_at)?;
        writeln!(page_file, "        <div>Updated: {}</div>", component.updated_at)?;
        writeln!(page_file, "    </div>")?;
        
        writeln!(page_file, "    <p>{}</p>", component.description)?;
        
        if !component.tags.is_empty() {
            writeln!(page_file, "    <div class=\"component-tags\">")?;
            for tag in &component.tags {
                writeln!(page_file, "        <span class=\"tag\">{}</span>", tag)?;
            }
            writeln!(page_file, "    </div>")?;
        }
        
        // Write responsibilities section
        if !component.responsibilities.is_empty() {
            writeln!(page_file, "    <h2>Responsibilities</h2>")?;
            writeln!(page_file, "    <ul>")?;
            for responsibility in &component.responsibilities {
                writeln!(page_file, "        <li>{}</li>", responsibility)?;
            }
            writeln!(page_file, "    </ul>")?;
        }
        
        // Write interfaces section
        if !component.interfaces.is_empty() {
            writeln!(page_file, "    <h2>Interfaces</h2>")?;
            writeln!(page_file, "    <ul>")?;
            for interface in &component.interfaces {
                writeln!(page_file, "        <li>{}</li>", interface)?;
            }
            writeln!(page_file, "    </ul>")?;
        }
        
        // Write dependencies section
        if !component.dependencies.is_empty() {
            writeln!(page_file, "    <h2>Dependencies</h2>")?;
            writeln!(page_file, "    <ul>")?;
            for dependency in &component.dependencies {
                writeln!(page_file, "        <li>{}</li>", dependency)?;
            }
            writeln!(page_file, "    </ul>")?;
        }
        
        // Write relationships section
        let incoming_relationships: Vec<&ComponentRelationship> = architecture.relationships.iter()
            .filter(|r| r.target == component.id)
            .collect();
        
        let outgoing_relationships: Vec<&ComponentRelationship> = architecture.relationships.iter()
            .filter(|r| r.source == component.id)
            .collect();
        
        if !incoming_relationships.is_empty() || !outgoing_relationships.is_empty() {
            writeln!(page_file, "    <h2>Relationships</h2>")?;
            
            if !incoming_relationships.is_empty() {
                writeln!(page_file, "    <h3>Incoming Relationships</h3>")?;
                
                for relationship in incoming_relationships {
                    let source_component = architecture.components.get(&relationship.source);
                    let source_name = source_component.map_or(&relationship.source, |c| &c.name);
                    
                    writeln!(page_file, "    <div class=\"relationship\">")?;
                    
                    if let Some(source) = source_component {
                        writeln!(page_file, "        <a href=\"component_{}_{}.html\">{}</a>", architecture.id, source.id, source.name)?;
                    } else {
                        writeln!(page_file, "        {}", source_name)?;
                    }
                    
                    if let Some(label) = &relationship.label {
                        writeln!(page_file, "        <span> {} </span>", label)?;
                    } else {
                        writeln!(page_file, "        <span> → </span>")?;
                    }
                    
                    writeln!(page_file, "        {}", component.name)?;
                    
                    if let Some(description) = &relationship.description {
                        writeln!(page_file, "        <div>{}</div>", description)?;
                    }
                    
                    writeln!(page_file, "    </div>")?;
                }
            }
            
            if !outgoing_relationships.is_empty() {
                writeln!(page_file, "    <h3>Outgoing Relationships</h3>")?;
                
                for relationship in outgoing_relationships {
                    let target_component = architecture.components.get(&relationship.target);
                    let target_name = target_component.map_or(&relationship.target, |c| &c.name);
                    
                    writeln!(page_file, "    <div class=\"relationship\">")?;
                    
                    writeln!(page_file, "        {}", component.name)?;
                    
                    if let Some(label) = &relationship.label {
                        writeln!(page_file, "        <span> {} </span>", label)?;
                    } else {
                        writeln!(page_file, "        <span> → </span>")?;
                    }
                    
                    if let Some(target) = target_component {
                        writeln!(page_file, "        <a href=\"component_{}_{}.html\">{}</a>", architecture.id, target.id, target.name)?;
                    } else {
                        writeln!(page_file, "        {}", target_name)?;
                    }
                    
                    if let Some(description) = &relationship.description {
                        writeln!(page_file, "        <div>{}</div>", description)?;
                    }
                    
                    writeln!(page_file, "    </div>")?;
                }
            }
        }
        
        // Write related diagrams section
        let related_diagrams: Vec<&Diagram> = architecture.diagrams.values()
            .filter(|d| d.related_components.contains(&component.id))
            .collect();
        
        if !related_diagrams.is_empty() {
            writeln!(page_file, "    <h2>Related Diagrams</h2>")?;
            
            for diagram in related_diagrams {
                writeln!(page_file, "    <div class=\"diagram\">")?;
                writeln!(page_file, "        <a href=\"diagram_{}_{}.html\">{}</a>", architecture.id, diagram.id, diagram.title)?;
                writeln!(page_file, "    </div>")?;
            }
        }
        
        writeln!(page_file, "    <p><a href=\"architecture_{}.html\">Back to architecture</a></p>", architecture.id)?;
        writeln!(page_file, "    <p><a href=\"index.html\">Back to index</a></p>")?;
        writeln!(page_file, "</body>")?;
        writeln!(page_file, "</html>")?;
        
        Ok(())
    }
    
    /// Generate diagram page
    fn generate_diagram_page(&self, output_dir: &Path, architecture: &SystemArchitecture, diagram: &Diagram) -> io::Result<()> {
        let page_path = output_dir.join(format!("diagram_{}_{}.html", architecture.id, diagram.id));
        let mut page_file = File::create(page_path)?;
        
        writeln!(page_file, "<!DOCTYPE html>")?;
        writeln!(page_file, "<html>")?;
        writeln!(page_file, "<head>")?;
        writeln!(page_file, "    <title>Diagram: {}</title>", diagram.title)?;
        writeln!(page_file, "    <meta charset=\"UTF-8\">")?;
        writeln!(page_file, "    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">")?;
        writeln!(page_file, "    <style>")?;
        writeln!(page_file, "        body {{ font-family: Arial, sans-serif; margin: 0; padding: 20px; }}")?;
        writeln!(page_file, "        h1, h2 {{ color: #333; }}")?;
        writeln!(page_file, "        .diagram-meta {{ color: #666; font-size: 0.9em; margin-bottom: 20px; }}")?;
        writeln!(page_file, "        .diagram-tags {{ margin-top: 10px; }}")?;
        writeln!(page_file, "        .tag {{ display: inline-block; background-color: #f0f0f0; padding: 3px 8px; margin-right: 5px; border-radius: 3px; font-size: 0.8em; }}")?;
        writeln!(page_file, "        .diagram-container {{ margin-top: 20px; border: 1px solid #ddd; padding: 20px; }}")?;
        writeln!(page_file, "        .diagram {{ width: 100%; height: 500px; }}")?;
        writeln!(page_file, "    </style>")?;
        
        // Include appropriate libraries based on diagram format
        match diagram.format {
            DiagramFormat::Mermaid => {
                writeln!(page_file, "    <script src=\"https://cdnjs.cloudflare.com/ajax/libs/mermaid/8.13.10/mermaid.min.js\"></script>")?;
            }
            DiagramFormat::PlantUML => {
                // PlantUML typically requires server-side rendering
                // For client-side, we could use a PlantUML proxy service
            }
            DiagramFormat::Graphviz => {
                writeln!(page_file, "    <script src=\"https://cdnjs.cloudflare.com/ajax/libs/viz.js/2.1.2/viz.js\"></script>")?;
                writeln!(page_file, "    <script src=\"https://cdnjs.cloudflare.com/ajax/libs/viz.js/2.1.2/full.render.js\"></script>")?;
            }
            _ => {}
        }
        
        writeln!(page_file, "</head>")?;
        writeln!(page_file, "<body>")?;
        writeln!(page_file, "    <h1>Diagram: {}</h1>", diagram.title)?;
        
        writeln!(page_file, "    <div class=\"diagram-meta\">")?;
        writeln!(page_file, "        <div>Type: {:?}</div>", diagram.diagram_type)?;
        writeln!(page_file, "        <div>Format: {:?}</div>", diagram.format)?;
        writeln!(page_file, "        <div>Version: {}</div>", diagram.version)?;
        if let Some(author) = &diagram.author {
            writeln!(page_file, "        <div>Author: {}</div>", author)?;
        }
        writeln!(page_file, "        <div>Created: {}</div>", diagram.created_at)?;
        writeln!(page_file, "        <div>Updated: {}</div>", diagram.updated_at)?;
        writeln!(page_file, "    </div>")?;
        
        writeln!(page_file, "    <p>{}</p>", diagram.description)?;
        
        if !diagram.tags.is_empty() {
            writeln!(page_file, "    <div class=\"diagram-tags\">")?;
            for tag in &diagram.tags {
                writeln!(page_file, "        <span class=\"tag\">{}</span>", tag)?;
            }
            writeln!(page_file, "    </div>")?;
        }
        
        // Write related components section
        if !diagram.related_components.is_empty() {
            writeln!(page_file, "    <h2>Related Components</h2>")?;
            writeln!(page_file, "    <ul>")?;
            
            for component_id in &diagram.related_components {
                if let Some(component) = architecture.components.get(component_id) {
                    writeln!(page_file, "        <li><a href=\"component_{}_{}.html\">{}</a></li>", architecture.id, component.id, component.name)?;
                } else {
                    writeln!(page_file, "        <li>{}</li>", component_id)?;
                }
            }
            
            writeln!(page_file, "    </ul>")?;
        }
        
        // Render diagram based on format
        writeln!(page_file, "    <div class=\"diagram-container\">")?;
        writeln!(page_file, "        <div class=\"diagram\" id=\"diagram\">")?;
        
        match diagram.format {
            DiagramFormat::Mermaid => {
                writeln!(page_file, "            <pre class=\"mermaid\">")?;
                writeln!(page_file, "{}", diagram.content)?;
                writeln!(page_file, "            </pre>")?;
                
                writeln!(page_file, "            <script>")?;
                writeln!(page_file, "                mermaid.initialize({{ startOnLoad: true }});")?;
                writeln!(page_file, "            </script>")?;
            }
            DiagramFormat::PlantUML => {
                // For PlantUML, we could use a proxy service or embed an image
                writeln!(page_file, "            <p>PlantUML diagram would be rendered here.</p>")?;
                writeln!(page_file, "            <pre>{}</pre>", diagram.content)?;
            }
            DiagramFormat::Graphviz => {
                writeln!(page_file, "            <div id=\"graphviz-diagram\"></div>")?;
                
                writeln!(page_file, "            <script>")?;
                writeln!(page_file, "                const dot = `{}`", diagram.content)?;
                writeln!(page_file, "                const viz = new Viz();")?;
                writeln!(page_file, "                viz.renderSVGElement(dot)")?;
                writeln!(page_file, "                    .then(element => {{")?;
                writeln!(page_file, "                        document.getElementById('graphviz-diagram').appendChild(element);")?;
                writeln!(page_file, "                    }})")?;
                writeln!(page_file, "                    .catch(error => {{")?;
                writeln!(page_file, "                        document.getElementById('graphviz-diagram').innerHTML = `<p>Error rendering diagram: ${{error.message}}</p>`;")?;
                writeln!(page_file, "                    }});")?;
                writeln!(page_file, "            </script>")?;
            }
            DiagramFormat::SVG => {
                // For SVG, we can directly embed the content
                writeln!(page_file, "{}", diagram.content)?;
            }
            _ => {
                // For custom formats, just show the content as text
                writeln!(page_file, "            <pre>{}</pre>", diagram.content)?;
            }
        }
        
        writeln!(page_file, "        </div>")?;
        writeln!(page_file, "    </div>")?;
        
        writeln!(page_file, "    <p><a href=\"architecture_{}.html\">Back to architecture</a></p>", architecture.id)?;
        writeln!(page_file, "    <p><a href=\"index.html\">Back to index</a></p>")?;
        writeln!(page_file, "</body>")?;
        writeln!(page_file, "</html>")?;
        
        Ok(())
    }
    
    /// Create example data
    pub fn create_example_data(&mut self, output_dir: &Path) -> io::Result<()> {
        info!("Creating example data");
        
        // Create architectures directory
        let architectures_dir = output_dir.join("architectures");
        fs::create_dir_all(&architectures_dir)?;
        
        // Create a sample architecture
        let mut architecture = SystemArchitecture::new(
            "evo-system",
            "Evo System Architecture",
            "The overall architecture of the Evo system, showing the main components and their relationships."
        )
        .with_version("1.0.0")
        .with_author("Evo Team");
        
        // Add components
        let frontend_component = Component::new(
            "frontend",
            "Frontend",
            "The user interface of the Evo system.",
            "UI"
        )
        .with_responsibility("Render the user interface")
        .with_responsibility("Handle user interactions")
        .with_responsibility("Communicate with the backend")
        .with_interface("User Interface")
        .with_dependency("Backend API")
        .with_tag("ui")
        .with_tag("react")
        .with_author("Evo Team");
        
        let backend_component = Component::new(
            "backend",
            "Backend",
            "The server-side component of the Evo system.",
            "Service"
        )
        .with_responsibility("Process requests from the frontend")
        .with_responsibility("Manage business logic")
        .with_responsibility("Communicate with the database")
        .with_interface("REST API")
        .with_dependency("Database")
        .with_tag("api")
        .with_tag("rust")
        .with_author("Evo Team");
        
        let database_component = Component::new(
            "database",
            "Database",
            "The data storage component of the Evo system.",
            "Storage"
        )
        .with_responsibility("Store and retrieve data")
        .with_responsibility("Ensure data integrity")
        .with_interface("SQL Interface")
        .with_tag("storage")
        .with_tag("sqlite")
        .with_author("Evo Team");
        
        let actor_system_component = Component::new(
            "actor-system",
            "Actor System",
            "The actor-based concurrency system of the Evo backend.",
            "Concurrency"
        )
        .with_responsibility("Manage concurrent operations")
        .with_responsibility("Handle message passing between actors")
        .with_dependency("Backend")
        .with_tag("concurrency")
        .with_tag("actors")
        .with_author("Evo Team");
        
        architecture.add_component(frontend_component);
        architecture.add_component(backend_component);
        architecture.add_component(database_component);
        architecture.add_component(actor_system_component);
        
        // Add relationships
        architecture.add_relationship(ComponentRelationship::new(
            "frontend",
            "backend",
            RelationshipType::Dependency
        ).with_label("uses").with_description("The frontend depends on the backend API"));
        
        architecture.add_relationship(ComponentRelationship::new(
            "backend",
            "database",
            RelationshipType::Dependency
        ).with_label("uses").with_description("The backend depends on the database for data storage"));
        
        architecture.add_relationship(ComponentRelationship::new(
            "backend",
            "actor-system",
            RelationshipType::Composition
        ).with_label("contains").with_description("The backend contains the actor system"));
        
        // Add diagrams
        let architecture_diagram = Diagram::new(
            "high-level",
            "High-Level Architecture",
            "A high-level overview of the Evo system architecture.",
            DiagramType::Architecture,
            DiagramFormat::Mermaid,
            "graph TD\n    A[Frontend] --> B[Backend]\n    B --> C[Database]\n    B --> D[Actor System]"
        )
        .with_tag("architecture")
        .with_tag("high-level")
        .with_related_component("frontend")
        .with_related_component("backend")
        .with_related_component("database")
        .with_related_component("actor-system")
        .with_author("Evo Team");
        
        let component_diagram = Diagram::new(
            "component-detail",
            "Component Detail",
            "A detailed view of the components in the Evo system.",
            DiagramType::Component,
            DiagramFormat::Mermaid,
            "classDiagram\n    class Frontend {\n        +render()\n        +handleEvent()\n    }\n    class Backend {\n        +processRequest()\n        +executeLogic()\n    }\n    class Database {\n        +query()\n        +update()\n    }\n    class ActorSystem {\n        +createActor()\n        +sendMessage()\n    }\n    Frontend --> Backend\n    Backend --> Database\n    Backend --> ActorSystem"
        )
        .with_tag("components")
        .with_tag("detail")
        .with_related_component("frontend")
        .with_related_component("backend")
        .with_related_component("database")
        .with_related_component("actor-system")
        .with_author("Evo Team");
        
        let data_flow_diagram = Diagram::new(
            "data-flow",
            "Data Flow",
            "The flow of data through the Evo system.",
            DiagramType::DataFlow,
            DiagramFormat::Mermaid,
            "graph LR\n    A[User] --> B[Frontend]\n    B --> C[Backend]\n    C --> D[Database]\n    D --> C\n    C --> B\n    B --> A"
        )
        .with_tag("data-flow")
        .with_related_component("frontend")
        .with_related_component("backend")
        .with_related_component("database")
        .with_author("Evo Team");
        
        architecture.add_diagram(architecture_diagram);
        architecture.add_diagram(component_diagram);
        architecture.add_diagram(data_flow_diagram);
        
        // Save architecture to file
        let architecture_path = architectures_dir.join("evo-system.json");
        let architecture_file = File::create(architecture_path)?;
        serde_json::to_writer_pretty(architecture_file, &architecture)?;
        
        self.architectures.insert(architecture.id.clone(), architecture);
        
        info!("Example data created successfully");
        Ok(())
    }
}

/// Global visual documentation generator
lazy_static::lazy_static! {
    static ref VISUAL_DOCS_GENERATOR: Arc<Mutex<VisualDocsGenerator>> = Arc::new(Mutex::new(
        VisualDocsGenerator::new(VisualDocsConfig::default(), DocsGenConfig::default())
    ));
}

/// Get the global visual documentation generator
pub fn get_visual_docs_generator() -> Arc<Mutex<VisualDocsGenerator>> {
    VISUAL_DOCS_GENERATOR.clone()
}

/// Configure visual documentation generation
pub fn configure(config: VisualDocsConfig, base_config: DocsGenConfig) {
    let mut generator = VISUAL_DOCS_GENERATOR.lock().unwrap();
    *generator = VisualDocsGenerator::new(config, base_config);
}

/// Generate visual documentation
pub fn generate_visual_docs(output_dir: &Path) -> io::Result<()> {
    let mut generator = VISUAL_DOCS_GENERATOR.lock().unwrap();
    
    // Check if visual documentation is enabled
    if !generator.config.enabled {
        info!("Visual documentation is disabled");
        return Ok(());
    }
    
    // Create example data if no architectures exist
    if generator.architectures.is_empty() {
        generator.create_example_data(output_dir)?;
    }
    
    // Generate visual documentation
    generator.generate_visual_docs(output_dir)?;
    
    info!("Visual documentation generated successfully");
    Ok(())
}

/// Initialize the visual documentation system
pub fn init() {
    info!("Initializing visual documentation system");
}