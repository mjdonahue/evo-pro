//! Interactive Examples and Tutorials
//!
//! This module provides tools for creating and managing interactive examples
//! and tutorials for the API documentation. It supports code playgrounds,
//! step-by-step tutorials, and interactive diagrams.

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use serde::{Serialize, Deserialize};
use tracing::{debug, info, warn, error};

use super::{DocItem, DocExample, DocsGenConfig};

/// Interactive example configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractiveConfig {
    /// Whether interactive examples are enabled
    pub enabled: bool,
    /// Whether to include a code playground
    pub include_playground: bool,
    /// Whether to include step-by-step tutorials
    pub include_tutorials: bool,
    /// Whether to include interactive diagrams
    pub include_diagrams: bool,
    /// Maximum number of examples per page
    pub max_examples_per_page: usize,
    /// Additional options
    pub options: HashMap<String, String>,
}

impl Default for InteractiveConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            include_playground: true,
            include_tutorials: true,
            include_diagrams: true,
            max_examples_per_page: 5,
            options: HashMap::new(),
        }
    }
}

/// Interactive example type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InteractiveExampleType {
    /// Code playground
    Playground,
    /// Step-by-step tutorial
    Tutorial,
    /// Interactive diagram
    Diagram,
}

/// Interactive example
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractiveExample {
    /// Example ID
    pub id: String,
    /// Example title
    pub title: String,
    /// Example description
    pub description: String,
    /// Example type
    pub example_type: InteractiveExampleType,
    /// Example content
    pub content: String,
    /// Example language (for code examples)
    pub language: Option<String>,
    /// Example dependencies
    pub dependencies: Vec<String>,
    /// Example tags
    pub tags: Vec<String>,
    /// Example difficulty level (1-5)
    pub difficulty: u8,
    /// Example estimated completion time (in minutes)
    pub estimated_time: u32,
    /// Example author
    pub author: Option<String>,
    /// Example creation date
    pub created_at: String,
    /// Example last updated date
    pub updated_at: String,
}

impl InteractiveExample {
    /// Create a new interactive example
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        description: impl Into<String>,
        example_type: InteractiveExampleType,
        content: impl Into<String>,
    ) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        
        Self {
            id: id.into(),
            title: title.into(),
            description: description.into(),
            example_type,
            content: content.into(),
            language: None,
            dependencies: Vec::new(),
            tags: Vec::new(),
            difficulty: 1,
            estimated_time: 10,
            author: None,
            created_at: now.clone(),
            updated_at: now,
        }
    }
    
    /// Set the example language
    pub fn with_language(mut self, language: impl Into<String>) -> Self {
        self.language = Some(language.into());
        self
    }
    
    /// Add a dependency to the example
    pub fn with_dependency(mut self, dependency: impl Into<String>) -> Self {
        self.dependencies.push(dependency.into());
        self
    }
    
    /// Add a tag to the example
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }
    
    /// Set the example difficulty level
    pub fn with_difficulty(mut self, difficulty: u8) -> Self {
        self.difficulty = difficulty.min(5).max(1);
        self
    }
    
    /// Set the example estimated completion time
    pub fn with_estimated_time(mut self, minutes: u32) -> Self {
        self.estimated_time = minutes;
        self
    }
    
    /// Set the example author
    pub fn with_author(mut self, author: impl Into<String>) -> Self {
        self.author = Some(author.into());
        self
    }
}

/// Tutorial step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TutorialStep {
    /// Step title
    pub title: String,
    /// Step description
    pub description: String,
    /// Step content
    pub content: String,
    /// Step code (if applicable)
    pub code: Option<String>,
    /// Expected output (if applicable)
    pub expected_output: Option<String>,
    /// Step hints
    pub hints: Vec<String>,
}

impl TutorialStep {
    /// Create a new tutorial step
    pub fn new(
        title: impl Into<String>,
        description: impl Into<String>,
        content: impl Into<String>,
    ) -> Self {
        Self {
            title: title.into(),
            description: description.into(),
            content: content.into(),
            code: None,
            expected_output: None,
            hints: Vec::new(),
        }
    }
    
    /// Set the step code
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }
    
    /// Set the expected output
    pub fn with_expected_output(mut self, output: impl Into<String>) -> Self {
        self.expected_output = Some(output.into());
        self
    }
    
    /// Add a hint to the step
    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hints.push(hint.into());
        self
    }
}

/// Tutorial
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tutorial {
    /// Tutorial ID
    pub id: String,
    /// Tutorial title
    pub title: String,
    /// Tutorial description
    pub description: String,
    /// Tutorial steps
    pub steps: Vec<TutorialStep>,
    /// Tutorial prerequisites
    pub prerequisites: Vec<String>,
    /// Tutorial tags
    pub tags: Vec<String>,
    /// Tutorial difficulty level (1-5)
    pub difficulty: u8,
    /// Tutorial estimated completion time (in minutes)
    pub estimated_time: u32,
    /// Tutorial author
    pub author: Option<String>,
    /// Tutorial creation date
    pub created_at: String,
    /// Tutorial last updated date
    pub updated_at: String,
}

impl Tutorial {
    /// Create a new tutorial
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        
        Self {
            id: id.into(),
            title: title.into(),
            description: description.into(),
            steps: Vec::new(),
            prerequisites: Vec::new(),
            tags: Vec::new(),
            difficulty: 1,
            estimated_time: 30,
            author: None,
            created_at: now.clone(),
            updated_at: now,
        }
    }
    
    /// Add a step to the tutorial
    pub fn with_step(mut self, step: TutorialStep) -> Self {
        self.steps.push(step);
        self
    }
    
    /// Add a prerequisite to the tutorial
    pub fn with_prerequisite(mut self, prerequisite: impl Into<String>) -> Self {
        self.prerequisites.push(prerequisite.into());
        self
    }
    
    /// Add a tag to the tutorial
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }
    
    /// Set the tutorial difficulty level
    pub fn with_difficulty(mut self, difficulty: u8) -> Self {
        self.difficulty = difficulty.min(5).max(1);
        self
    }
    
    /// Set the tutorial estimated completion time
    pub fn with_estimated_time(mut self, minutes: u32) -> Self {
        self.estimated_time = minutes;
        self
    }
    
    /// Set the tutorial author
    pub fn with_author(mut self, author: impl Into<String>) -> Self {
        self.author = Some(author.into());
        self
    }
}

/// Interactive examples generator
#[derive(Debug)]
pub struct InteractiveExamplesGenerator {
    /// Configuration
    pub config: InteractiveConfig,
    /// Base documentation generator configuration
    pub base_config: DocsGenConfig,
    /// Interactive examples
    pub examples: HashMap<String, InteractiveExample>,
    /// Tutorials
    pub tutorials: HashMap<String, Tutorial>,
}

impl InteractiveExamplesGenerator {
    /// Create a new interactive examples generator
    pub fn new(config: InteractiveConfig, base_config: DocsGenConfig) -> Self {
        Self {
            config,
            base_config,
            examples: HashMap::new(),
            tutorials: HashMap::new(),
        }
    }
    
    /// Load examples from a directory
    pub fn load_examples(&mut self, dir: &Path) -> io::Result<()> {
        info!("Loading interactive examples from {}", dir.display());
        
        let examples_dir = dir.join("examples");
        if examples_dir.exists() && examples_dir.is_dir() {
            for entry in fs::read_dir(examples_dir)? {
                let entry = entry?;
                let path = entry.path();
                
                if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                    let mut file = File::open(&path)?;
                    let mut contents = String::new();
                    file.read_to_string(&mut contents)?;
                    
                    match serde_json::from_str::<InteractiveExample>(&contents) {
                        Ok(example) => {
                            self.examples.insert(example.id.clone(), example);
                        }
                        Err(err) => {
                            warn!("Failed to parse example file {}: {}", path.display(), err);
                        }
                    }
                }
            }
        }
        
        info!("Loaded {} interactive examples", self.examples.len());
        Ok(())
    }
    
    /// Load tutorials from a directory
    pub fn load_tutorials(&mut self, dir: &Path) -> io::Result<()> {
        info!("Loading tutorials from {}", dir.display());
        
        let tutorials_dir = dir.join("tutorials");
        if tutorials_dir.exists() && tutorials_dir.is_dir() {
            for entry in fs::read_dir(tutorials_dir)? {
                let entry = entry?;
                let path = entry.path();
                
                if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                    let mut file = File::open(&path)?;
                    let mut contents = String::new();
                    file.read_to_string(&mut contents)?;
                    
                    match serde_json::from_str::<Tutorial>(&contents) {
                        Ok(tutorial) => {
                            self.tutorials.insert(tutorial.id.clone(), tutorial);
                        }
                        Err(err) => {
                            warn!("Failed to parse tutorial file {}: {}", path.display(), err);
                        }
                    }
                }
            }
        }
        
        info!("Loaded {} tutorials", self.tutorials.len());
        Ok(())
    }
    
    /// Generate interactive examples
    pub fn generate_interactive_examples(&self, output_dir: &Path) -> io::Result<()> {
        if !self.config.enabled {
            info!("Interactive examples are disabled");
            return Ok(());
        }
        
        info!("Generating interactive examples");
        
        // Create output directory
        let examples_dir = output_dir.join("interactive");
        fs::create_dir_all(&examples_dir)?;
        
        // Generate index page
        self.generate_index(&examples_dir)?;
        
        // Generate example pages
        for example in self.examples.values() {
            match example.example_type {
                InteractiveExampleType::Playground => {
                    if self.config.include_playground {
                        self.generate_playground(&examples_dir, example)?;
                    }
                }
                InteractiveExampleType::Diagram => {
                    if self.config.include_diagrams {
                        self.generate_diagram(&examples_dir, example)?;
                    }
                }
                _ => {}
            }
        }
        
        // Generate tutorial pages
        if self.config.include_tutorials {
            for tutorial in self.tutorials.values() {
                self.generate_tutorial(&examples_dir, tutorial)?;
            }
        }
        
        info!("Interactive examples generated successfully");
        Ok(())
    }
    
    /// Generate index page
    fn generate_index(&self, output_dir: &Path) -> io::Result<()> {
        let index_path = output_dir.join("index.html");
        let mut index_file = File::create(index_path)?;
        
        writeln!(index_file, "<!DOCTYPE html>")?;
        writeln!(index_file, "<html>")?;
        writeln!(index_file, "<head>")?;
        writeln!(index_file, "    <title>Interactive Examples and Tutorials</title>")?;
        writeln!(index_file, "    <meta charset=\"UTF-8\">")?;
        writeln!(index_file, "    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">")?;
        writeln!(index_file, "    <style>")?;
        writeln!(index_file, "        body {{ font-family: Arial, sans-serif; margin: 0; padding: 20px; }}")?;
        writeln!(index_file, "        h1, h2 {{ color: #333; }}")?;
        writeln!(index_file, "        .example {{ margin-bottom: 20px; padding: 15px; border: 1px solid #ddd; border-radius: 5px; }}")?;
        writeln!(index_file, "        .example h3 {{ margin-top: 0; }}")?;
        writeln!(index_file, "        .example-meta {{ color: #666; font-size: 0.9em; margin-bottom: 10px; }}")?;
        writeln!(index_file, "        .example-tags {{ margin-top: 10px; }}")?;
        writeln!(index_file, "        .tag {{ display: inline-block; background-color: #f0f0f0; padding: 3px 8px; margin-right: 5px; border-radius: 3px; font-size: 0.8em; }}")?;
        writeln!(index_file, "        .difficulty {{ display: inline-block; margin-right: 15px; }}")?;
        writeln!(index_file, "        .difficulty-1 {{ color: #4caf50; }}")?;
        writeln!(index_file, "        .difficulty-2 {{ color: #8bc34a; }}")?;
        writeln!(index_file, "        .difficulty-3 {{ color: #ffc107; }}")?;
        writeln!(index_file, "        .difficulty-4 {{ color: #ff9800; }}")?;
        writeln!(index_file, "        .difficulty-5 {{ color: #f44336; }}")?;
        writeln!(index_file, "    </style>")?;
        writeln!(index_file, "</head>")?;
        writeln!(index_file, "<body>")?;
        writeln!(index_file, "    <h1>Interactive Examples and Tutorials</h1>")?;
        
        // Write playgrounds section
        if self.config.include_playground {
            writeln!(index_file, "    <h2>Code Playgrounds</h2>")?;
            
            let playgrounds: Vec<&InteractiveExample> = self.examples.values()
                .filter(|e| e.example_type == InteractiveExampleType::Playground)
                .collect();
            
            if playgrounds.is_empty() {
                writeln!(index_file, "    <p>No code playgrounds available.</p>")?;
            } else {
                for example in playgrounds {
                    writeln!(index_file, "    <div class=\"example\">")?;
                    writeln!(index_file, "        <h3><a href=\"playground_{}.html\">{}</a></h3>", example.id, example.title)?;
                    writeln!(index_file, "        <div class=\"example-meta\">")?;
                    writeln!(index_file, "            <span class=\"difficulty difficulty-{}\">Difficulty: {}/5</span>", example.difficulty, example.difficulty)?;
                    writeln!(index_file, "            <span class=\"time\">Estimated time: {} minutes</span>", example.estimated_time)?;
                    if let Some(author) = &example.author {
                        writeln!(index_file, "            <span class=\"author\">Author: {}</span>", author)?;
                    }
                    writeln!(index_file, "        </div>")?;
                    writeln!(index_file, "        <p>{}</p>", example.description)?;
                    if !example.tags.is_empty() {
                        writeln!(index_file, "        <div class=\"example-tags\">")?;
                        for tag in &example.tags {
                            writeln!(index_file, "            <span class=\"tag\">{}</span>", tag)?;
                        }
                        writeln!(index_file, "        </div>")?;
                    }
                    writeln!(index_file, "    </div>")?;
                }
            }
        }
        
        // Write tutorials section
        if self.config.include_tutorials {
            writeln!(index_file, "    <h2>Tutorials</h2>")?;
            
            if self.tutorials.is_empty() {
                writeln!(index_file, "    <p>No tutorials available.</p>")?;
            } else {
                for tutorial in self.tutorials.values() {
                    writeln!(index_file, "    <div class=\"example\">")?;
                    writeln!(index_file, "        <h3><a href=\"tutorial_{}.html\">{}</a></h3>", tutorial.id, tutorial.title)?;
                    writeln!(index_file, "        <div class=\"example-meta\">")?;
                    writeln!(index_file, "            <span class=\"difficulty difficulty-{}\">Difficulty: {}/5</span>", tutorial.difficulty, tutorial.difficulty)?;
                    writeln!(index_file, "            <span class=\"time\">Estimated time: {} minutes</span>", tutorial.estimated_time)?;
                    if let Some(author) = &tutorial.author {
                        writeln!(index_file, "            <span class=\"author\">Author: {}</span>", author)?;
                    }
                    writeln!(index_file, "        </div>")?;
                    writeln!(index_file, "        <p>{}</p>", tutorial.description)?;
                    if !tutorial.prerequisites.is_empty() {
                        writeln!(index_file, "        <p><strong>Prerequisites:</strong> {}</p>", tutorial.prerequisites.join(", "))?;
                    }
                    if !tutorial.tags.is_empty() {
                        writeln!(index_file, "        <div class=\"example-tags\">")?;
                        for tag in &tutorial.tags {
                            writeln!(index_file, "            <span class=\"tag\">{}</span>", tag)?;
                        }
                        writeln!(index_file, "        </div>")?;
                    }
                    writeln!(index_file, "    </div>")?;
                }
            }
        }
        
        // Write diagrams section
        if self.config.include_diagrams {
            writeln!(index_file, "    <h2>Interactive Diagrams</h2>")?;
            
            let diagrams: Vec<&InteractiveExample> = self.examples.values()
                .filter(|e| e.example_type == InteractiveExampleType::Diagram)
                .collect();
            
            if diagrams.is_empty() {
                writeln!(index_file, "    <p>No interactive diagrams available.</p>")?;
            } else {
                for example in diagrams {
                    writeln!(index_file, "    <div class=\"example\">")?;
                    writeln!(index_file, "        <h3><a href=\"diagram_{}.html\">{}</a></h3>", example.id, example.title)?;
                    writeln!(index_file, "        <p>{}</p>", example.description)?;
                    if !example.tags.is_empty() {
                        writeln!(index_file, "        <div class=\"example-tags\">")?;
                        for tag in &example.tags {
                            writeln!(index_file, "            <span class=\"tag\">{}</span>", tag)?;
                        }
                        writeln!(index_file, "        </div>")?;
                    }
                    writeln!(index_file, "    </div>")?;
                }
            }
        }
        
        writeln!(index_file, "    <p><a href=\"../index.html\">Back to documentation</a></p>")?;
        writeln!(index_file, "</body>")?;
        writeln!(index_file, "</html>")?;
        
        Ok(())
    }
    
    /// Generate a playground page
    fn generate_playground(&self, output_dir: &Path, example: &InteractiveExample) -> io::Result<()> {
        let page_path = output_dir.join(format!("playground_{}.html", example.id));
        let mut page_file = File::create(page_path)?;
        
        writeln!(page_file, "<!DOCTYPE html>")?;
        writeln!(page_file, "<html>")?;
        writeln!(page_file, "<head>")?;
        writeln!(page_file, "    <title>Playground: {}</title>", example.title)?;
        writeln!(page_file, "    <meta charset=\"UTF-8\">")?;
        writeln!(page_file, "    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">")?;
        writeln!(page_file, "    <style>")?;
        writeln!(page_file, "        body {{ font-family: Arial, sans-serif; margin: 0; padding: 20px; }}")?;
        writeln!(page_file, "        h1, h2 {{ color: #333; }}")?;
        writeln!(page_file, "        .playground {{ display: flex; flex-direction: column; gap: 10px; margin-top: 20px; }}")?;
        writeln!(page_file, "        .editor {{ height: 300px; border: 1px solid #ddd; }}")?;
        writeln!(page_file, "        .controls {{ display: flex; gap: 10px; }}")?;
        writeln!(page_file, "        .controls button {{ padding: 8px 16px; background-color: #4CAF50; color: white; border: none; cursor: pointer; }}")?;
        writeln!(page_file, "        .controls button:hover {{ background-color: #45a049; }}")?;
        writeln!(page_file, "        .output {{ padding: 10px; background-color: #f5f5f5; border: 1px solid #ddd; min-height: 100px; }}")?;
        writeln!(page_file, "        .example-meta {{ color: #666; font-size: 0.9em; margin-bottom: 10px; }}")?;
        writeln!(page_file, "        .example-tags {{ margin-top: 10px; }}")?;
        writeln!(page_file, "        .tag {{ display: inline-block; background-color: #f0f0f0; padding: 3px 8px; margin-right: 5px; border-radius: 3px; font-size: 0.8em; }}")?;
        writeln!(page_file, "    </style>")?;
        writeln!(page_file, "    <script src=\"https://cdnjs.cloudflare.com/ajax/libs/ace/1.4.12/ace.js\"></script>")?;
        writeln!(page_file, "</head>")?;
        writeln!(page_file, "<body>")?;
        writeln!(page_file, "    <h1>Playground: {}</h1>", example.title)?;
        
        writeln!(page_file, "    <div class=\"example-meta\">")?;
        writeln!(page_file, "        <span class=\"difficulty\">Difficulty: {}/5</span>", example.difficulty)?;
        writeln!(page_file, "        <span class=\"time\">Estimated time: {} minutes</span>", example.estimated_time)?;
        if let Some(author) = &example.author {
            writeln!(page_file, "        <span class=\"author\">Author: {}</span>", author)?;
        }
        writeln!(page_file, "    </div>")?;
        
        writeln!(page_file, "    <p>{}</p>", example.description)?;
        
        if !example.tags.is_empty() {
            writeln!(page_file, "    <div class=\"example-tags\">")?;
            for tag in &example.tags {
                writeln!(page_file, "        <span class=\"tag\">{}</span>", tag)?;
            }
            writeln!(page_file, "    </div>")?;
        }
        
        writeln!(page_file, "    <div class=\"playground\">")?;
        writeln!(page_file, "        <div id=\"editor\" class=\"editor\">{}</div>", example.content)?;
        writeln!(page_file, "        <div class=\"controls\">")?;
        writeln!(page_file, "            <button id=\"run-button\">Run</button>")?;
        writeln!(page_file, "            <button id=\"reset-button\">Reset</button>")?;
        writeln!(page_file, "        </div>")?;
        writeln!(page_file, "        <div id=\"output\" class=\"output\">Output will appear here...</div>")?;
        writeln!(page_file, "    </div>")?;
        
        writeln!(page_file, "    <script>")?;
        writeln!(page_file, "        // Initialize the editor")?;
        writeln!(page_file, "        const editor = ace.edit(\"editor\");")?;
        if let Some(language) = &example.language {
            writeln!(page_file, "        editor.session.setMode(\"ace/mode/{}\");", language)?;
        }
        writeln!(page_file, "        editor.setTheme(\"ace/theme/monokai\");")?;
        writeln!(page_file, "        editor.setFontSize(14);")?;
        writeln!(page_file, "        ")?;
        writeln!(page_file, "        // Store the original content for reset")?;
        writeln!(page_file, "        const originalContent = editor.getValue();")?;
        writeln!(page_file, "        ")?;
        writeln!(page_file, "        // Set up the run button")?;
        writeln!(page_file, "        document.getElementById(\"run-button\").addEventListener(\"click\", function() {{")?;
        writeln!(page_file, "            const code = editor.getValue();")?;
        writeln!(page_file, "            const output = document.getElementById(\"output\");")?;
        writeln!(page_file, "            ")?;
        writeln!(page_file, "            // In a real implementation, this would execute the code")?;
        writeln!(page_file, "            // For now, we'll just display the code")?;
        writeln!(page_file, "            output.innerHTML = \"<pre>\" + code + \"</pre>\";")?;
        writeln!(page_file, "        }});")?;
        writeln!(page_file, "        ")?;
        writeln!(page_file, "        // Set up the reset button")?;
        writeln!(page_file, "        document.getElementById(\"reset-button\").addEventListener(\"click\", function() {{")?;
        writeln!(page_file, "            editor.setValue(originalContent);")?;
        writeln!(page_file, "            editor.clearSelection();")?;
        writeln!(page_file, "            document.getElementById(\"output\").innerHTML = \"Output will appear here...\";")?;
        writeln!(page_file, "        }});")?;
        writeln!(page_file, "    </script>")?;
        
        writeln!(page_file, "    <p><a href=\"index.html\">Back to examples</a></p>")?;
        writeln!(page_file, "</body>")?;
        writeln!(page_file, "</html>")?;
        
        Ok(())
    }
    
    /// Generate a tutorial page
    fn generate_tutorial(&self, output_dir: &Path, tutorial: &Tutorial) -> io::Result<()> {
        let page_path = output_dir.join(format!("tutorial_{}.html", tutorial.id));
        let mut page_file = File::create(page_path)?;
        
        writeln!(page_file, "<!DOCTYPE html>")?;
        writeln!(page_file, "<html>")?;
        writeln!(page_file, "<head>")?;
        writeln!(page_file, "    <title>Tutorial: {}</title>", tutorial.title)?;
        writeln!(page_file, "    <meta charset=\"UTF-8\">")?;
        writeln!(page_file, "    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">")?;
        writeln!(page_file, "    <style>")?;
        writeln!(page_file, "        body {{ font-family: Arial, sans-serif; margin: 0; padding: 20px; }}")?;
        writeln!(page_file, "        h1, h2, h3 {{ color: #333; }}")?;
        writeln!(page_file, "        .tutorial-meta {{ color: #666; font-size: 0.9em; margin-bottom: 20px; }}")?;
        writeln!(page_file, "        .tutorial-tags {{ margin-top: 10px; }}")?;
        writeln!(page_file, "        .tag {{ display: inline-block; background-color: #f0f0f0; padding: 3px 8px; margin-right: 5px; border-radius: 3px; font-size: 0.8em; }}")?;
        writeln!(page_file, "        .steps {{ counter-reset: step; }}")?;
        writeln!(page_file, "        .step {{ margin-bottom: 30px; padding: 20px; border: 1px solid #ddd; border-radius: 5px; }}")?;
        writeln!(page_file, "        .step h3 {{ margin-top: 0; }}")?;
        writeln!(page_file, "        .step h3::before {{ counter-increment: step; content: \"Step \" counter(step) \": \"; }}")?;
        writeln!(page_file, "        .step-content {{ margin-bottom: 15px; }}")?;
        writeln!(page_file, "        .step-code {{ background-color: #f5f5f5; padding: 10px; border-radius: 5px; overflow-x: auto; }}")?;
        writeln!(page_file, "        .step-code pre {{ margin: 0; }}")?;
        writeln!(page_file, "        .step-output {{ background-color: #f0f0f0; padding: 10px; border-radius: 5px; margin-top: 10px; }}")?;
        writeln!(page_file, "        .step-hints {{ margin-top: 15px; }}")?;
        writeln!(page_file, "        .step-hint {{ background-color: #fff3cd; padding: 10px; border-radius: 5px; margin-top: 5px; }}")?;
        writeln!(page_file, "        .navigation {{ display: flex; justify-content: space-between; margin-top: 20px; }}")?;
        writeln!(page_file, "        .navigation button {{ padding: 8px 16px; background-color: #4CAF50; color: white; border: none; cursor: pointer; }}")?;
        writeln!(page_file, "        .navigation button:hover {{ background-color: #45a049; }}")?;
        writeln!(page_file, "        .navigation button:disabled {{ background-color: #cccccc; cursor: not-allowed; }}")?;
        writeln!(page_file, "    </style>")?;
        writeln!(page_file, "</head>")?;
        writeln!(page_file, "<body>")?;
        writeln!(page_file, "    <h1>Tutorial: {}</h1>", tutorial.title)?;
        
        writeln!(page_file, "    <div class=\"tutorial-meta\">")?;
        writeln!(page_file, "        <span class=\"difficulty\">Difficulty: {}/5</span>", tutorial.difficulty)?;
        writeln!(page_file, "        <span class=\"time\">Estimated time: {} minutes</span>", tutorial.estimated_time)?;
        if let Some(author) = &tutorial.author {
            writeln!(page_file, "        <span class=\"author\">Author: {}</span>", author)?;
        }
        writeln!(page_file, "    </div>")?;
        
        writeln!(page_file, "    <p>{}</p>", tutorial.description)?;
        
        if !tutorial.prerequisites.is_empty() {
            writeln!(page_file, "    <h2>Prerequisites</h2>")?;
            writeln!(page_file, "    <ul>")?;
            for prerequisite in &tutorial.prerequisites {
                writeln!(page_file, "        <li>{}</li>", prerequisite)?;
            }
            writeln!(page_file, "    </ul>")?;
        }
        
        if !tutorial.tags.is_empty() {
            writeln!(page_file, "    <div class=\"tutorial-tags\">")?;
            for tag in &tutorial.tags {
                writeln!(page_file, "        <span class=\"tag\">{}</span>", tag)?;
            }
            writeln!(page_file, "    </div>")?;
        }
        
        writeln!(page_file, "    <h2>Steps</h2>")?;
        writeln!(page_file, "    <div class=\"steps\" id=\"steps\">")?;
        
        for (i, step) in tutorial.steps.iter().enumerate() {
            let step_id = format!("step-{}", i + 1);
            let display = if i == 0 { "block" } else { "none" };
            
            writeln!(page_file, "        <div class=\"step\" id=\"{}\" style=\"display: {};\">", step_id, display)?;
            writeln!(page_file, "            <h3>{}</h3>", step.title)?;
            writeln!(page_file, "            <div class=\"step-content\">")?;
            writeln!(page_file, "                <p>{}</p>", step.description)?;
            writeln!(page_file, "                <div>{}</div>", step.content)?;
            writeln!(page_file, "            </div>")?;
            
            if let Some(code) = &step.code {
                writeln!(page_file, "            <div class=\"step-code\">")?;
                writeln!(page_file, "                <pre>{}</pre>", code)?;
                writeln!(page_file, "            </div>")?;
            }
            
            if let Some(output) = &step.expected_output {
                writeln!(page_file, "            <div class=\"step-output\">")?;
                writeln!(page_file, "                <h4>Expected Output:</h4>")?;
                writeln!(page_file, "                <pre>{}</pre>", output)?;
                writeln!(page_file, "            </div>")?;
            }
            
            if !step.hints.is_empty() {
                writeln!(page_file, "            <div class=\"step-hints\">")?;
                writeln!(page_file, "                <h4>Hints:</h4>")?;
                for (j, hint) in step.hints.iter().enumerate() {
                    let hint_id = format!("hint-{}-{}", i + 1, j + 1);
                    writeln!(page_file, "                <div>")?;
                    writeln!(page_file, "                    <button onclick=\"document.getElementById('{}').style.display='block';\">Show Hint {}</button>", hint_id, j + 1)?;
                    writeln!(page_file, "                    <div id=\"{}\" class=\"step-hint\" style=\"display: none;\">{}</div>", hint_id, hint)?;
                    writeln!(page_file, "                </div>")?;
                }
                writeln!(page_file, "            </div>")?;
            }
            
            writeln!(page_file, "        </div>")?;
        }
        
        writeln!(page_file, "    </div>")?;
        
        writeln!(page_file, "    <div class=\"navigation\">")?;
        writeln!(page_file, "        <button id=\"prev-button\" onclick=\"navigateStep(-1)\" disabled>Previous</button>")?;
        writeln!(page_file, "        <span id=\"step-counter\">Step 1 of {}</span>", tutorial.steps.len())?;
        writeln!(page_file, "        <button id=\"next-button\" onclick=\"navigateStep(1)\"{}>Next</button>", if tutorial.steps.len() <= 1 { " disabled" } else { "" })?;
        writeln!(page_file, "    </div>")?;
        
        writeln!(page_file, "    <script>")?;
        writeln!(page_file, "        let currentStep = 1;")?;
        writeln!(page_file, "        const totalSteps = {};", tutorial.steps.len())?;
        writeln!(page_file, "        ")?;
        writeln!(page_file, "        function navigateStep(direction) {{")?;
        writeln!(page_file, "            // Hide current step")?;
        writeln!(page_file, "            document.getElementById(`step-${{currentStep}}`).style.display = 'none';")?;
        writeln!(page_file, "            ")?;
        writeln!(page_file, "            // Update current step")?;
        writeln!(page_file, "            currentStep += direction;")?;
        writeln!(page_file, "            ")?;
        writeln!(page_file, "            // Show new current step")?;
        writeln!(page_file, "            document.getElementById(`step-${{currentStep}}`).style.display = 'block';")?;
        writeln!(page_file, "            ")?;
        writeln!(page_file, "            // Update counter")?;
        writeln!(page_file, "            document.getElementById('step-counter').textContent = `Step ${{currentStep}} of ${{totalSteps}}`;")?;
        writeln!(page_file, "            ")?;
        writeln!(page_file, "            // Update button states")?;
        writeln!(page_file, "            document.getElementById('prev-button').disabled = currentStep === 1;")?;
        writeln!(page_file, "            document.getElementById('next-button').disabled = currentStep === totalSteps;")?;
        writeln!(page_file, "        }}")?;
        writeln!(page_file, "    </script>")?;
        
        writeln!(page_file, "    <p><a href=\"index.html\">Back to examples</a></p>")?;
        writeln!(page_file, "</body>")?;
        writeln!(page_file, "</html>")?;
        
        Ok(())
    }
    
    /// Generate a diagram page
    fn generate_diagram(&self, output_dir: &Path, example: &InteractiveExample) -> io::Result<()> {
        let page_path = output_dir.join(format!("diagram_{}.html", example.id));
        let mut page_file = File::create(page_path)?;
        
        writeln!(page_file, "<!DOCTYPE html>")?;
        writeln!(page_file, "<html>")?;
        writeln!(page_file, "<head>")?;
        writeln!(page_file, "    <title>Interactive Diagram: {}</title>", example.title)?;
        writeln!(page_file, "    <meta charset=\"UTF-8\">")?;
        writeln!(page_file, "    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">")?;
        writeln!(page_file, "    <style>")?;
        writeln!(page_file, "        body {{ font-family: Arial, sans-serif; margin: 0; padding: 20px; }}")?;
        writeln!(page_file, "        h1, h2 {{ color: #333; }}")?;
        writeln!(page_file, "        .diagram-container {{ margin-top: 20px; border: 1px solid #ddd; padding: 20px; }}")?;
        writeln!(page_file, "        .diagram {{ width: 100%; height: 500px; }}")?;
        writeln!(page_file, "        .example-meta {{ color: #666; font-size: 0.9em; margin-bottom: 10px; }}")?;
        writeln!(page_file, "        .example-tags {{ margin-top: 10px; }}")?;
        writeln!(page_file, "        .tag {{ display: inline-block; background-color: #f0f0f0; padding: 3px 8px; margin-right: 5px; border-radius: 3px; font-size: 0.8em; }}")?;
        writeln!(page_file, "    </style>")?;
        writeln!(page_file, "    <script src=\"https://cdnjs.cloudflare.com/ajax/libs/mermaid/8.13.10/mermaid.min.js\"></script>")?;
        writeln!(page_file, "</head>")?;
        writeln!(page_file, "<body>")?;
        writeln!(page_file, "    <h1>Interactive Diagram: {}</h1>", example.title)?;
        
        writeln!(page_file, "    <div class=\"example-meta\">")?;
        if let Some(author) = &example.author {
            writeln!(page_file, "        <span class=\"author\">Author: {}</span>", author)?;
        }
        writeln!(page_file, "    </div>")?;
        
        writeln!(page_file, "    <p>{}</p>", example.description)?;
        
        if !example.tags.is_empty() {
            writeln!(page_file, "    <div class=\"example-tags\">")?;
            for tag in &example.tags {
                writeln!(page_file, "        <span class=\"tag\">{}</span>", tag)?;
            }
            writeln!(page_file, "    </div>")?;
        }
        
        writeln!(page_file, "    <div class=\"diagram-container\">")?;
        writeln!(page_file, "        <div class=\"diagram\">")?;
        writeln!(page_file, "            <pre class=\"mermaid\">")?;
        writeln!(page_file, "{}", example.content)?;
        writeln!(page_file, "            </pre>")?;
        writeln!(page_file, "        </div>")?;
        writeln!(page_file, "    </div>")?;
        
        writeln!(page_file, "    <script>")?;
        writeln!(page_file, "        mermaid.initialize({{ startOnLoad: true }});")?;
        writeln!(page_file, "    </script>")?;
        
        writeln!(page_file, "    <p><a href=\"index.html\">Back to examples</a></p>")?;
        writeln!(page_file, "</body>")?;
        writeln!(page_file, "</html>")?;
        
        Ok(())
    }
    
    /// Create example data
    pub fn create_example_data(&mut self, output_dir: &Path) -> io::Result<()> {
        info!("Creating example data");
        
        // Create examples directory
        let examples_dir = output_dir.join("examples");
        fs::create_dir_all(&examples_dir)?;
        
        // Create tutorials directory
        let tutorials_dir = output_dir.join("tutorials");
        fs::create_dir_all(&tutorials_dir)?;
        
        // Create a sample playground example
        let playground_example = InteractiveExample::new(
            "rust-basics",
            "Rust Basics Playground",
            "A simple playground to experiment with Rust basics.",
            InteractiveExampleType::Playground,
            "fn main() {\n    println!(\"Hello, world!\");\n}"
        )
        .with_language("rust")
        .with_tag("rust")
        .with_tag("beginner")
        .with_difficulty(1)
        .with_estimated_time(15)
        .with_author("Evo Team");
        
        let playground_path = examples_dir.join("rust-basics.json");
        let playground_file = File::create(playground_path)?;
        serde_json::to_writer_pretty(playground_file, &playground_example)?;
        
        self.examples.insert(playground_example.id.clone(), playground_example);
        
        // Create a sample diagram example
        let diagram_example = InteractiveExample::new(
            "actor-system",
            "Actor System Architecture",
            "An interactive diagram showing the actor system architecture.",
            InteractiveExampleType::Diagram,
            "graph TD\n    A[Client] --> B[API Gateway]\n    B --> C[Actor System]\n    C --> D[Database Actor]\n    C --> E[File System Actor]\n    C --> F[Network Actor]"
        )
        .with_tag("architecture")
        .with_tag("actors")
        .with_author("Evo Team");
        
        let diagram_path = examples_dir.join("actor-system.json");
        let diagram_file = File::create(diagram_path)?;
        serde_json::to_writer_pretty(diagram_file, &diagram_example)?;
        
        self.examples.insert(diagram_example.id.clone(), diagram_example);
        
        // Create a sample tutorial
        let tutorial = Tutorial::new(
            "getting-started",
            "Getting Started with Evo",
            "A step-by-step tutorial to get started with the Evo application."
        )
        .with_step(TutorialStep::new(
            "Installation",
            "Install the Evo application on your system.",
            "Follow these steps to install Evo:\n\n1. Download the installer from the website\n2. Run the installer\n3. Follow the on-screen instructions"
        ))
        .with_step(TutorialStep::new(
            "Create Your First Project",
            "Create your first project in Evo.",
            "1. Open Evo\n2. Click on 'New Project'\n3. Enter a name for your project\n4. Select a template\n5. Click 'Create'"
        )
        .with_code("// Example project configuration\n{\n    \"name\": \"My First Project\",\n    \"template\": \"basic\",\n    \"version\": \"1.0.0\"\n}")
        .with_hint("Choose a descriptive name for your project"))
        .with_step(TutorialStep::new(
            "Explore the Interface",
            "Explore the Evo interface and learn about its features.",
            "The Evo interface consists of several key areas:\n\n- Sidebar: Contains project navigation\n- Editor: Main editing area\n- Console: Shows output and logs\n- Status Bar: Shows current status and tools"
        ))
        .with_prerequisite("Basic knowledge of programming")
        .with_tag("beginner")
        .with_tag("tutorial")
        .with_difficulty(1)
        .with_estimated_time(30)
        .with_author("Evo Team");
        
        let tutorial_path = tutorials_dir.join("getting-started.json");
        let tutorial_file = File::create(tutorial_path)?;
        serde_json::to_writer_pretty(tutorial_file, &tutorial)?;
        
        self.tutorials.insert(tutorial.id.clone(), tutorial);
        
        info!("Example data created successfully");
        Ok(())
    }
}

/// Global interactive examples generator
lazy_static::lazy_static! {
    static ref INTERACTIVE_EXAMPLES_GENERATOR: Arc<Mutex<InteractiveExamplesGenerator>> = Arc::new(Mutex::new(
        InteractiveExamplesGenerator::new(InteractiveConfig::default(), DocsGenConfig::default())
    ));
}

/// Get the global interactive examples generator
pub fn get_interactive_examples_generator() -> Arc<Mutex<InteractiveExamplesGenerator>> {
    INTERACTIVE_EXAMPLES_GENERATOR.clone()
}

/// Configure interactive examples generation
pub fn configure(config: InteractiveConfig, base_config: DocsGenConfig) {
    let mut generator = INTERACTIVE_EXAMPLES_GENERATOR.lock().unwrap();
    *generator = InteractiveExamplesGenerator::new(config, base_config);
}

/// Generate interactive examples
pub fn generate_interactive_examples(output_dir: &Path) -> io::Result<()> {
    let mut generator = INTERACTIVE_EXAMPLES_GENERATOR.lock().unwrap();
    
    // Check if interactive examples are enabled
    if !generator.config.enabled {
        info!("Interactive examples are disabled");
        return Ok(());
    }
    
    // Create example data if no examples exist
    if generator.examples.is_empty() && generator.tutorials.is_empty() {
        generator.create_example_data(output_dir)?;
    }
    
    // Generate interactive examples
    generator.generate_interactive_examples(output_dir)?;
    
    info!("Interactive examples generated successfully");
    Ok(())
}

/// Initialize the interactive examples system
pub fn init() {
    info!("Initializing interactive examples system");
}