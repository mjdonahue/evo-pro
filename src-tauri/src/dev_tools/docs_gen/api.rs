//! Automated API Documentation Generation
//!
//! This module provides tools for automatically generating API documentation
//! from the codebase. It uses Rust's built-in documentation comments and type
//! information to generate comprehensive API documentation.

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use serde::{Serialize, Deserialize};
use tracing::{debug, info, warn, error};

use super::{DocItem, DocItemType, DocVisibility, DocExample, DocParameter, DocsGenConfig};

/// API documentation generation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiDocsConfig {
    /// Whether to include private items
    pub include_private: bool,
    /// Whether to include internal items
    pub include_internal: bool,
    /// Whether to include deprecated items
    pub include_deprecated: bool,
    /// Whether to include examples
    pub include_examples: bool,
    /// Whether to include source code
    pub include_source: bool,
    /// Whether to generate search index
    pub generate_search_index: bool,
    /// Whether to generate method index
    pub generate_method_index: bool,
    /// Output format (html, markdown, json)
    pub output_format: String,
    /// Additional options
    pub options: HashMap<String, String>,
}

impl Default for ApiDocsConfig {
    fn default() -> Self {
        Self {
            include_private: false,
            include_internal: true,
            include_deprecated: true,
            include_examples: true,
            include_source: true,
            generate_search_index: true,
            generate_method_index: true,
            output_format: "html".to_string(),
            options: HashMap::new(),
        }
    }
}

/// API documentation generator
#[derive(Debug)]
pub struct ApiDocsGenerator {
    /// Configuration
    pub config: ApiDocsConfig,
    /// Base documentation generator configuration
    pub base_config: DocsGenConfig,
    /// Documentation items
    pub items: Vec<DocItem>,
    /// Module hierarchy
    pub modules: HashMap<String, Vec<DocItem>>,
    /// Type index
    pub type_index: HashMap<String, DocItem>,
    /// Method index
    pub method_index: HashMap<String, Vec<DocItem>>,
}

impl ApiDocsGenerator {
    /// Create a new API documentation generator
    pub fn new(config: ApiDocsConfig, base_config: DocsGenConfig) -> Self {
        Self {
            config,
            base_config,
            items: Vec::new(),
            modules: HashMap::new(),
            type_index: HashMap::new(),
            method_index: HashMap::new(),
        }
    }
    
    /// Scan source directories for API documentation
    pub fn scan_sources(&mut self) -> io::Result<()> {
        info!("Scanning source directories for API documentation");
        
        for source_dir in &self.base_config.source_dirs {
            let source_path = Path::new(source_dir);
            if source_path.exists() && source_path.is_dir() {
                self.scan_directory(source_path)?;
            } else {
                warn!("Source directory does not exist: {}", source_dir);
            }
        }
        
        // Build module hierarchy
        self.build_module_hierarchy();
        
        // Build type index
        self.build_type_index();
        
        // Build method index if enabled
        if self.config.generate_method_index {
            self.build_method_index();
        }
        
        info!("Found {} documentation items", self.items.len());
        Ok(())
    }
    
    /// Scan a directory for API documentation
    fn scan_directory(&mut self, dir: &Path) -> io::Result<()> {
        debug!("Scanning directory: {}", dir.display());
        
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                self.scan_directory(&path)?;
            } else if path.is_file() {
                if let Some(extension) = path.extension() {
                    if extension == "rs" {
                        self.scan_file(&path)?;
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Scan a file for API documentation
    fn scan_file(&mut self, file: &Path) -> io::Result<()> {
        debug!("Scanning file: {}", file.display());
        
        let mut file_content = String::new();
        let mut file_handle = File::open(file)?;
        file_handle.read_to_string(&mut file_content)?;
        
        // Parse the file content to extract API documentation
        let items = parse_rust_file(&file_content, file);
        
        // Filter items based on configuration
        let filtered_items = items.into_iter()
            .filter(|item| {
                // Filter by visibility
                match item.visibility {
                    DocVisibility::Public => true,
                    DocVisibility::Internal => self.config.include_internal,
                    DocVisibility::Private => self.config.include_private,
                }
            })
            .filter(|item| {
                // Filter by deprecation status
                !item.deprecated || self.config.include_deprecated
            })
            .collect::<Vec<_>>();
        
        // Add filtered items to the collection
        self.items.extend(filtered_items);
        
        Ok(())
    }
    
    /// Build module hierarchy
    fn build_module_hierarchy(&mut self) {
        debug!("Building module hierarchy");
        
        // Group items by module
        for item in &self.items {
            if let Some(module_path) = extract_module_path(&item.path) {
                let module_items = self.modules.entry(module_path.clone()).or_insert_with(Vec::new);
                module_items.push(item.clone());
            }
        }
    }
    
    /// Build type index
    fn build_type_index(&mut self) {
        debug!("Building type index");
        
        // Index items by type name
        for item in &self.items {
            match item.item_type {
                DocItemType::Struct | DocItemType::Enum | DocItemType::Trait | DocItemType::TypeAlias => {
                    self.type_index.insert(item.name.clone(), item.clone());
                }
                _ => {}
            }
        }
    }
    
    /// Build method index
    fn build_method_index(&mut self) {
        debug!("Building method index");
        
        // Group methods by name
        for item in &self.items {
            if item.item_type == DocItemType::Method || item.item_type == DocItemType::Function {
                let methods = self.method_index.entry(item.name.clone()).or_insert_with(Vec::new);
                methods.push(item.clone());
            }
        }
    }
    
    /// Generate API documentation
    pub fn generate_docs(&self) -> io::Result<()> {
        info!("Generating API documentation");
        
        // Create output directory
        let output_dir = Path::new(&self.base_config.output_dir).join("api");
        fs::create_dir_all(&output_dir)?;
        
        // Generate documentation based on output format
        match self.config.output_format.as_str() {
            "html" => self.generate_html_docs(&output_dir)?,
            "markdown" => self.generate_markdown_docs(&output_dir)?,
            "json" => self.generate_json_docs(&output_dir)?,
            _ => {
                warn!("Unsupported output format: {}", self.config.output_format);
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("Unsupported output format: {}", self.config.output_format)
                ));
            }
        }
        
        // Generate search index if enabled
        if self.config.generate_search_index {
            self.generate_search_index(&output_dir)?;
        }
        
        info!("API documentation generated successfully");
        Ok(())
    }
    
    /// Generate HTML API documentation
    fn generate_html_docs(&self, output_dir: &Path) -> io::Result<()> {
        debug!("Generating HTML API documentation");
        
        // Generate index page
        self.generate_html_index(output_dir)?;
        
        // Generate module pages
        for (module_path, items) in &self.modules {
            self.generate_html_module(output_dir, module_path, items)?;
        }
        
        // Generate item pages
        for item in &self.items {
            match item.item_type {
                DocItemType::Struct => self.generate_html_struct(output_dir, item)?,
                DocItemType::Enum => self.generate_html_enum(output_dir, item)?,
                DocItemType::Trait => self.generate_html_trait(output_dir, item)?,
                DocItemType::Function => self.generate_html_function(output_dir, item)?,
                _ => {}
            }
        }
        
        Ok(())
    }
    
    /// Generate HTML index page
    fn generate_html_index(&self, output_dir: &Path) -> io::Result<()> {
        let index_path = output_dir.join("index.html");
        let mut index_file = File::create(index_path)?;
        
        writeln!(index_file, "<!DOCTYPE html>")?;
        writeln!(index_file, "<html>")?;
        writeln!(index_file, "<head>")?;
        writeln!(index_file, "    <title>API Documentation</title>")?;
        writeln!(index_file, "    <meta charset=\"UTF-8\">")?;
        writeln!(index_file, "    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">")?;
        writeln!(index_file, "    <style>")?;
        writeln!(index_file, "        body {{ font-family: Arial, sans-serif; margin: 0; padding: 20px; }}")?;
        writeln!(index_file, "        h1 {{ color: #333; }}")?;
        writeln!(index_file, "        .module {{ margin-bottom: 10px; }}")?;
        writeln!(index_file, "        .module a {{ text-decoration: none; color: #0077cc; }}")?;
        writeln!(index_file, "        .module a:hover {{ text-decoration: underline; }}")?;
        writeln!(index_file, "    </style>")?;
        writeln!(index_file, "</head>")?;
        writeln!(index_file, "<body>")?;
        writeln!(index_file, "    <h1>API Documentation</h1>")?;
        
        // List modules
        writeln!(index_file, "    <h2>Modules</h2>")?;
        
        // Sort modules by path
        let mut module_paths: Vec<&String> = self.modules.keys().collect();
        module_paths.sort();
        
        for module_path in module_paths {
            let module_name = module_path.split("::").last().unwrap_or(module_path);
            let module_file = format!("{}.html", module_path.replace("::", "_"));
            
            writeln!(index_file, "    <div class=\"module\">")?;
            writeln!(index_file, "        <a href=\"{}\">{}</a>", module_file, module_path)?;
            writeln!(index_file, "    </div>")?;
        }
        
        writeln!(index_file, "</body>")?;
        writeln!(index_file, "</html>")?;
        
        Ok(())
    }
    
    /// Generate HTML module page
    fn generate_html_module(&self, output_dir: &Path, module_path: &str, items: &[DocItem]) -> io::Result<()> {
        let module_file = format!("{}.html", module_path.replace("::", "_"));
        let module_path = output_dir.join(module_file);
        let mut module_file = File::create(module_path)?;
        
        writeln!(module_file, "<!DOCTYPE html>")?;
        writeln!(module_file, "<html>")?;
        writeln!(module_file, "<head>")?;
        writeln!(module_file, "    <title>Module {}</title>", module_path)?;
        writeln!(module_file, "    <meta charset=\"UTF-8\">")?;
        writeln!(module_file, "    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">")?;
        writeln!(module_file, "    <style>")?;
        writeln!(module_file, "        body {{ font-family: Arial, sans-serif; margin: 0; padding: 20px; }}")?;
        writeln!(module_file, "        h1 {{ color: #333; }}")?;
        writeln!(module_file, "        .item {{ margin-bottom: 10px; }}")?;
        writeln!(module_file, "        .item a {{ text-decoration: none; color: #0077cc; }}")?;
        writeln!(module_file, "        .item a:hover {{ text-decoration: underline; }}")?;
        writeln!(module_file, "        .item-type {{ color: #666; font-size: 0.8em; }}")?;
        writeln!(module_file, "    </style>")?;
        writeln!(module_file, "</head>")?;
        writeln!(module_file, "<body>")?;
        writeln!(module_file, "    <h1>Module {}</h1>", module_path)?;
        
        // Group items by type
        let mut structs = Vec::new();
        let mut enums = Vec::new();
        let mut traits = Vec::new();
        let mut functions = Vec::new();
        let mut type_aliases = Vec::new();
        let mut constants = Vec::new();
        let mut macros = Vec::new();
        
        for item in items {
            match item.item_type {
                DocItemType::Struct => structs.push(item),
                DocItemType::Enum => enums.push(item),
                DocItemType::Trait => traits.push(item),
                DocItemType::Function => functions.push(item),
                DocItemType::TypeAlias => type_aliases.push(item),
                DocItemType::Constant => constants.push(item),
                DocItemType::Macro => macros.push(item),
                _ => {}
            }
        }
        
        // Write structs
        if !structs.is_empty() {
            writeln!(module_file, "    <h2>Structs</h2>")?;
            for item in structs {
                let item_file = format!("{}.html", item.name);
                
                writeln!(module_file, "    <div class=\"item\">")?;
                writeln!(module_file, "        <a href=\"{}\">{}</a>", item_file, item.name)?;
                writeln!(module_file, "        <span class=\"item-type\">struct</span>")?;
                writeln!(module_file, "    </div>")?;
            }
        }
        
        // Write enums
        if !enums.is_empty() {
            writeln!(module_file, "    <h2>Enums</h2>")?;
            for item in enums {
                let item_file = format!("{}.html", item.name);
                
                writeln!(module_file, "    <div class=\"item\">")?;
                writeln!(module_file, "        <a href=\"{}\">{}</a>", item_file, item.name)?;
                writeln!(module_file, "        <span class=\"item-type\">enum</span>")?;
                writeln!(module_file, "    </div>")?;
            }
        }
        
        // Write traits
        if !traits.is_empty() {
            writeln!(module_file, "    <h2>Traits</h2>")?;
            for item in traits {
                let item_file = format!("{}.html", item.name);
                
                writeln!(module_file, "    <div class=\"item\">")?;
                writeln!(module_file, "        <a href=\"{}\">{}</a>", item_file, item.name)?;
                writeln!(module_file, "        <span class=\"item-type\">trait</span>")?;
                writeln!(module_file, "    </div>")?;
            }
        }
        
        // Write functions
        if !functions.is_empty() {
            writeln!(module_file, "    <h2>Functions</h2>")?;
            for item in functions {
                let item_file = format!("{}.html", item.name);
                
                writeln!(module_file, "    <div class=\"item\">")?;
                writeln!(module_file, "        <a href=\"{}\">{}</a>", item_file, item.name)?;
                writeln!(module_file, "        <span class=\"item-type\">function</span>")?;
                writeln!(module_file, "    </div>")?;
            }
        }
        
        // Write type aliases
        if !type_aliases.is_empty() {
            writeln!(module_file, "    <h2>Type Aliases</h2>")?;
            for item in type_aliases {
                let item_file = format!("{}.html", item.name);
                
                writeln!(module_file, "    <div class=\"item\">")?;
                writeln!(module_file, "        <a href=\"{}\">{}</a>", item_file, item.name)?;
                writeln!(module_file, "        <span class=\"item-type\">type</span>")?;
                writeln!(module_file, "    </div>")?;
            }
        }
        
        // Write constants
        if !constants.is_empty() {
            writeln!(module_file, "    <h2>Constants</h2>")?;
            for item in constants {
                let item_file = format!("{}.html", item.name);
                
                writeln!(module_file, "    <div class=\"item\">")?;
                writeln!(module_file, "        <a href=\"{}\">{}</a>", item_file, item.name)?;
                writeln!(module_file, "        <span class=\"item-type\">constant</span>")?;
                writeln!(module_file, "    </div>")?;
            }
        }
        
        // Write macros
        if !macros.is_empty() {
            writeln!(module_file, "    <h2>Macros</h2>")?;
            for item in macros {
                let item_file = format!("{}.html", item.name);
                
                writeln!(module_file, "    <div class=\"item\">")?;
                writeln!(module_file, "        <a href=\"{}\">{}</a>", item_file, item.name)?;
                writeln!(module_file, "        <span class=\"item-type\">macro</span>")?;
                writeln!(module_file, "    </div>")?;
            }
        }
        
        writeln!(module_file, "    <p><a href=\"index.html\">Back to index</a></p>")?;
        writeln!(module_file, "</body>")?;
        writeln!(module_file, "</html>")?;
        
        Ok(())
    }
    
    /// Generate HTML struct page
    fn generate_html_struct(&self, output_dir: &Path, item: &DocItem) -> io::Result<()> {
        let struct_path = output_dir.join(format!("{}.html", item.name));
        let mut struct_file = File::create(struct_path)?;
        
        writeln!(struct_file, "<!DOCTYPE html>")?;
        writeln!(struct_file, "<html>")?;
        writeln!(struct_file, "<head>")?;
        writeln!(struct_file, "    <title>Struct {}</title>", item.name)?;
        writeln!(struct_file, "    <meta charset=\"UTF-8\">")?;
        writeln!(struct_file, "    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">")?;
        writeln!(struct_file, "    <style>")?;
        writeln!(struct_file, "        body {{ font-family: Arial, sans-serif; margin: 0; padding: 20px; }}")?;
        writeln!(struct_file, "        h1 {{ color: #333; }}")?;
        writeln!(struct_file, "        .deprecated {{ color: #d9534f; }}")?;
        writeln!(struct_file, "        .documentation {{ margin-top: 20px; }}")?;
        writeln!(struct_file, "        .example {{ background-color: #f5f5f5; padding: 10px; margin-top: 10px; }}")?;
        writeln!(struct_file, "        .example pre {{ margin: 0; }}")?;
        writeln!(struct_file, "        .method {{ margin-top: 20px; border-top: 1px solid #ddd; padding-top: 10px; }}")?;
        writeln!(struct_file, "    </style>")?;
        writeln!(struct_file, "</head>")?;
        writeln!(struct_file, "<body>")?;
        
        // Write header
        if item.deprecated {
            writeln!(struct_file, "    <h1 class=\"deprecated\">Struct {} (Deprecated)</h1>", item.name)?;
            if let Some(message) = &item.deprecation_message {
                writeln!(struct_file, "    <p class=\"deprecated\">{}</p>", message)?;
            }
        } else {
            writeln!(struct_file, "    <h1>Struct {}</h1>", item.name)?;
        }
        
        // Write documentation
        writeln!(struct_file, "    <div class=\"documentation\">")?;
        writeln!(struct_file, "        <p>{}</p>", item.documentation)?;
        writeln!(struct_file, "    </div>")?;
        
        // Write examples if enabled
        if self.config.include_examples && !item.examples.is_empty() {
            writeln!(struct_file, "    <h2>Examples</h2>")?;
            for example in &item.examples {
                writeln!(struct_file, "    <div class=\"example\">")?;
                if let Some(title) = &example.title {
                    writeln!(struct_file, "        <h3>{}</h3>", title)?;
                }
                writeln!(struct_file, "        <pre>{}</pre>", example.code)?;
                if let Some(output) = &example.output {
                    writeln!(struct_file, "        <h4>Output:</h4>")?;
                    writeln!(struct_file, "        <pre>{}</pre>", output)?;
                }
                writeln!(struct_file, "    </div>")?;
            }
        }
        
        // Write methods
        let methods: Vec<&DocItem> = item.associated_items.iter()
            .filter(|i| i.item_type == DocItemType::Method)
            .collect();
        
        if !methods.is_empty() {
            writeln!(struct_file, "    <h2>Methods</h2>")?;
            for method in methods {
                writeln!(struct_file, "    <div class=\"method\" id=\"method.{}\">")?;
                writeln!(struct_file, "        <h3>{}</h3>", method.name)?;
                writeln!(struct_file, "        <div class=\"documentation\">")?;
                writeln!(struct_file, "            <p>{}</p>", method.documentation)?;
                writeln!(struct_file, "        </div>")?;
                
                // Write parameters
                if !method.parameters.is_empty() {
                    writeln!(struct_file, "        <h4>Parameters</h4>")?;
                    writeln!(struct_file, "        <ul>")?;
                    for param in &method.parameters {
                        writeln!(struct_file, "            <li>")?;
                        writeln!(struct_file, "                <code>{}: {}</code>", param.name, param.param_type)?;
                        writeln!(struct_file, "                <p>{}</p>", param.documentation)?;
                        writeln!(struct_file, "            </li>")?;
                    }
                    writeln!(struct_file, "        </ul>")?;
                }
                
                // Write return type and documentation
                if let Some(return_type) = &method.return_type {
                    writeln!(struct_file, "        <h4>Returns</h4>")?;
                    writeln!(struct_file, "        <p><code>{}</code></p>", return_type)?;
                    if let Some(return_doc) = &method.return_documentation {
                        writeln!(struct_file, "        <p>{}</p>", return_doc)?;
                    }
                }
                
                // Write examples if enabled
                if self.config.include_examples && !method.examples.is_empty() {
                    writeln!(struct_file, "        <h4>Examples</h4>")?;
                    for example in &method.examples {
                        writeln!(struct_file, "        <div class=\"example\">")?;
                        if let Some(title) = &example.title {
                            writeln!(struct_file, "            <h5>{}</h5>", title)?;
                        }
                        writeln!(struct_file, "            <pre>{}</pre>", example.code)?;
                        if let Some(output) = &example.output {
                            writeln!(struct_file, "            <h6>Output:</h6>")?;
                            writeln!(struct_file, "            <pre>{}</pre>", output)?;
                        }
                        writeln!(struct_file, "        </div>")?;
                    }
                }
                
                writeln!(struct_file, "    </div>")?;
            }
        }
        
        // Write source code if enabled
        if self.config.include_source {
            writeln!(struct_file, "    <h2>Source</h2>")?;
            writeln!(struct_file, "    <p>Source file: {}</p>", item.source_file)?;
        }
        
        writeln!(struct_file, "    <p><a href=\"index.html\">Back to index</a></p>")?;
        writeln!(struct_file, "</body>")?;
        writeln!(struct_file, "</html>")?;
        
        Ok(())
    }
    
    /// Generate HTML enum page
    fn generate_html_enum(&self, output_dir: &Path, item: &DocItem) -> io::Result<()> {
        // Similar to generate_html_struct, but for enums
        let enum_path = output_dir.join(format!("{}.html", item.name));
        let mut enum_file = File::create(enum_path)?;
        
        writeln!(enum_file, "<!DOCTYPE html>")?;
        writeln!(enum_file, "<html>")?;
        writeln!(enum_file, "<head>")?;
        writeln!(enum_file, "    <title>Enum {}</title>", item.name)?;
        writeln!(enum_file, "    <meta charset=\"UTF-8\">")?;
        writeln!(enum_file, "    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">")?;
        writeln!(enum_file, "    <style>")?;
        writeln!(enum_file, "        body {{ font-family: Arial, sans-serif; margin: 0; padding: 20px; }}")?;
        writeln!(enum_file, "        h1 {{ color: #333; }}")?;
        writeln!(enum_file, "        .documentation {{ margin-top: 20px; }}")?;
        writeln!(enum_file, "    </style>")?;
        writeln!(enum_file, "</head>")?;
        writeln!(enum_file, "<body>")?;
        writeln!(enum_file, "    <h1>Enum {}</h1>", item.name)?;
        writeln!(enum_file, "    <div class=\"documentation\">")?;
        writeln!(enum_file, "        <p>{}</p>", item.documentation)?;
        writeln!(enum_file, "    </div>")?;
        writeln!(enum_file, "    <p><a href=\"index.html\">Back to index</a></p>")?;
        writeln!(enum_file, "</body>")?;
        writeln!(enum_file, "</html>")?;
        
        Ok(())
    }
    
    /// Generate HTML trait page
    fn generate_html_trait(&self, output_dir: &Path, item: &DocItem) -> io::Result<()> {
        // Similar to generate_html_struct, but for traits
        let trait_path = output_dir.join(format!("{}.html", item.name));
        let mut trait_file = File::create(trait_path)?;
        
        writeln!(trait_file, "<!DOCTYPE html>")?;
        writeln!(trait_file, "<html>")?;
        writeln!(trait_file, "<head>")?;
        writeln!(trait_file, "    <title>Trait {}</title>", item.name)?;
        writeln!(trait_file, "    <meta charset=\"UTF-8\">")?;
        writeln!(trait_file, "    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">")?;
        writeln!(trait_file, "    <style>")?;
        writeln!(trait_file, "        body {{ font-family: Arial, sans-serif; margin: 0; padding: 20px; }}")?;
        writeln!(trait_file, "        h1 {{ color: #333; }}")?;
        writeln!(trait_file, "        .documentation {{ margin-top: 20px; }}")?;
        writeln!(trait_file, "    </style>")?;
        writeln!(trait_file, "</head>")?;
        writeln!(trait_file, "<body>")?;
        writeln!(trait_file, "    <h1>Trait {}</h1>", item.name)?;
        writeln!(trait_file, "    <div class=\"documentation\">")?;
        writeln!(trait_file, "        <p>{}</p>", item.documentation)?;
        writeln!(trait_file, "    </div>")?;
        writeln!(trait_file, "    <p><a href=\"index.html\">Back to index</a></p>")?;
        writeln!(trait_file, "</body>")?;
        writeln!(trait_file, "</html>")?;
        
        Ok(())
    }
    
    /// Generate HTML function page
    fn generate_html_function(&self, output_dir: &Path, item: &DocItem) -> io::Result<()> {
        // Similar to generate_html_struct, but for functions
        let function_path = output_dir.join(format!("{}.html", item.name));
        let mut function_file = File::create(function_path)?;
        
        writeln!(function_file, "<!DOCTYPE html>")?;
        writeln!(function_file, "<html>")?;
        writeln!(function_file, "<head>")?;
        writeln!(function_file, "    <title>Function {}</title>", item.name)?;
        writeln!(function_file, "    <meta charset=\"UTF-8\">")?;
        writeln!(function_file, "    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">")?;
        writeln!(function_file, "    <style>")?;
        writeln!(function_file, "        body {{ font-family: Arial, sans-serif; margin: 0; padding: 20px; }}")?;
        writeln!(function_file, "        h1 {{ color: #333; }}")?;
        writeln!(function_file, "        .documentation {{ margin-top: 20px; }}")?;
        writeln!(function_file, "    </style>")?;
        writeln!(function_file, "</head>")?;
        writeln!(function_file, "<body>")?;
        writeln!(function_file, "    <h1>Function {}</h1>", item.name)?;
        writeln!(function_file, "    <div class=\"documentation\">")?;
        writeln!(function_file, "        <p>{}</p>", item.documentation)?;
        writeln!(function_file, "    </div>")?;
        writeln!(function_file, "    <p><a href=\"index.html\">Back to index</a></p>")?;
        writeln!(function_file, "</body>")?;
        writeln!(function_file, "</html>")?;
        
        Ok(())
    }
    
    /// Generate Markdown API documentation
    fn generate_markdown_docs(&self, output_dir: &Path) -> io::Result<()> {
        debug!("Generating Markdown API documentation");
        
        // Generate index page
        self.generate_markdown_index(output_dir)?;
        
        // Generate module pages
        for (module_path, items) in &self.modules {
            self.generate_markdown_module(output_dir, module_path, items)?;
        }
        
        Ok(())
    }
    
    /// Generate Markdown index page
    fn generate_markdown_index(&self, output_dir: &Path) -> io::Result<()> {
        let index_path = output_dir.join("README.md");
        let mut index_file = File::create(index_path)?;
        
        writeln!(index_file, "# API Documentation")?;
        writeln!(index_file)?;
        
        // List modules
        writeln!(index_file, "## Modules")?;
        writeln!(index_file)?;
        
        // Sort modules by path
        let mut module_paths: Vec<&String> = self.modules.keys().collect();
        module_paths.sort();
        
        for module_path in module_paths {
            let module_file = format!("{}.md", module_path.replace("::", "_"));
            
            writeln!(index_file, "- [{}]({})", module_path, module_file)?;
        }
        
        Ok(())
    }
    
    /// Generate Markdown module page
    fn generate_markdown_module(&self, output_dir: &Path, module_path: &str, items: &[DocItem]) -> io::Result<()> {
        let module_file = format!("{}.md", module_path.replace("::", "_"));
        let module_path = output_dir.join(module_file);
        let mut module_file = File::create(module_path)?;
        
        writeln!(module_file, "# Module {}", module_path)?;
        writeln!(module_file)?;
        
        // Group items by type
        let mut structs = Vec::new();
        let mut enums = Vec::new();
        let mut traits = Vec::new();
        let mut functions = Vec::new();
        let mut type_aliases = Vec::new();
        let mut constants = Vec::new();
        let mut macros = Vec::new();
        
        for item in items {
            match item.item_type {
                DocItemType::Struct => structs.push(item),
                DocItemType::Enum => enums.push(item),
                DocItemType::Trait => traits.push(item),
                DocItemType::Function => functions.push(item),
                DocItemType::TypeAlias => type_aliases.push(item),
                DocItemType::Constant => constants.push(item),
                DocItemType::Macro => macros.push(item),
                _ => {}
            }
        }
        
        // Write structs
        if !structs.is_empty() {
            writeln!(module_file, "## Structs")?;
            writeln!(module_file)?;
            for item in structs {
                writeln!(module_file, "- {}", item.name)?;
            }
            writeln!(module_file)?;
        }
        
        // Write enums
        if !enums.is_empty() {
            writeln!(module_file, "## Enums")?;
            writeln!(module_file)?;
            for item in enums {
                writeln!(module_file, "- {}", item.name)?;
            }
            writeln!(module_file)?;
        }
        
        // Write traits
        if !traits.is_empty() {
            writeln!(module_file, "## Traits")?;
            writeln!(module_file)?;
            for item in traits {
                writeln!(module_file, "- {}", item.name)?;
            }
            writeln!(module_file)?;
        }
        
        // Write functions
        if !functions.is_empty() {
            writeln!(module_file, "## Functions")?;
            writeln!(module_file)?;
            for item in functions {
                writeln!(module_file, "- {}", item.name)?;
            }
            writeln!(module_file)?;
        }
        
        writeln!(module_file, "[Back to index](README.md)")?;
        
        Ok(())
    }
    
    /// Generate JSON API documentation
    fn generate_json_docs(&self, output_dir: &Path) -> io::Result<()> {
        debug!("Generating JSON API documentation");
        
        // Create a serializable structure for the API documentation
        #[derive(Serialize)]
        struct ApiDocs {
            modules: HashMap<String, Vec<DocItem>>,
            items: Vec<DocItem>,
        }
        
        let api_docs = ApiDocs {
            modules: self.modules.clone(),
            items: self.items.clone(),
        };
        
        // Write to file
        let json_path = output_dir.join("api.json");
        let json_file = File::create(json_path)?;
        
        serde_json::to_writer_pretty(json_file, &api_docs)?;
        
        Ok(())
    }
    
    /// Generate search index
    fn generate_search_index(&self, output_dir: &Path) -> io::Result<()> {
        debug!("Generating search index");
        
        // Create a serializable structure for the search index
        #[derive(Serialize)]
        struct SearchIndex {
            items: Vec<SearchItem>,
        }
        
        #[derive(Serialize)]
        struct SearchItem {
            name: String,
            path: String,
            description: String,
            item_type: String,
        }
        
        let mut search_items = Vec::new();
        
        // Add items to search index
        for item in &self.items {
            let item_type = match item.item_type {
                DocItemType::Module => "module",
                DocItemType::Struct => "struct",
                DocItemType::Enum => "enum",
                DocItemType::Trait => "trait",
                DocItemType::Function => "function",
                DocItemType::Method => "method",
                DocItemType::Constant => "constant",
                DocItemType::TypeAlias => "type",
                DocItemType::Macro => "macro",
            };
            
            // Extract a short description from the documentation
            let description = item.documentation.lines().next().unwrap_or("").to_string();
            
            search_items.push(SearchItem {
                name: item.name.clone(),
                path: item.path.clone(),
                description,
                item_type: item_type.to_string(),
            });
        }
        
        let search_index = SearchIndex {
            items: search_items,
        };
        
        // Write to file
        let index_path = output_dir.join("search-index.json");
        let index_file = File::create(index_path)?;
        
        serde_json::to_writer_pretty(index_file, &search_index)?;
        
        Ok(())
    }
}

/// Parse a Rust file to extract API documentation
fn parse_rust_file(content: &str, file: &Path) -> Vec<DocItem> {
    // In a real implementation, this would use a Rust parser to extract documentation
    // For now, we'll just create a simple representation based on the file name
    
    let mut items = Vec::new();
    
    if let Some(file_name) = file.file_name() {
        if let Some(file_name_str) = file_name.to_str() {
            if file_name_str == "mod.rs" || file_name_str == "lib.rs" {
                // This is a module file
                let module_name = file.parent()
                    .and_then(|p| p.file_name())
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown");
                
                let doc_item = DocItem {
                    name: module_name.to_string(),
                    item_type: DocItemType::Module,
                    visibility: DocVisibility::Public,
                    path: format!("crate::{}", module_name),
                    documentation: extract_module_doc(content),
                    source_file: file.to_string_lossy().to_string(),
                    line: 1,
                    deprecated: false,
                    deprecation_message: None,
                    examples: extract_examples(content),
                    parameters: Vec::new(),
                    return_type: None,
                    return_documentation: None,
                    associated_items: Vec::new(),
                    parent: None,
                };
                
                items.push(doc_item);
            } else if file_name_str.ends_with(".rs") {
                // This is a regular Rust file
                let item_name = file_name_str.trim_end_matches(".rs");
                
                let doc_item = DocItem {
                    name: item_name.to_string(),
                    item_type: guess_item_type(content),
                    visibility: guess_visibility(content),
                    path: format!("crate::{}", item_name),
                    documentation: extract_item_doc(content),
                    source_file: file.to_string_lossy().to_string(),
                    line: 1,
                    deprecated: is_deprecated(content),
                    deprecation_message: extract_deprecation_message(content),
                    examples: extract_examples(content),
                    parameters: extract_parameters(content),
                    return_type: extract_return_type(content),
                    return_documentation: extract_return_doc(content),
                    associated_items: Vec::new(),
                    parent: None,
                };
                
                items.push(doc_item);
            }
        }
    }
    
    items
}

/// Extract module path from item path
fn extract_module_path(path: &str) -> Option<String> {
    let parts: Vec<&str> = path.split("::").collect();
    if parts.len() > 1 {
        Some(parts[..parts.len() - 1].join("::"))
    } else {
        None
    }
}

/// Extract module documentation from file content
fn extract_module_doc(content: &str) -> String {
    // In a real implementation, this would parse the file content and extract the module documentation
    // For now, we'll just return a placeholder
    "Module documentation would be extracted here.".to_string()
}

/// Extract item documentation from file content
fn extract_item_doc(content: &str) -> String {
    // In a real implementation, this would parse the file content and extract the item documentation
    // For now, we'll just return a placeholder
    "Item documentation would be extracted here.".to_string()
}

/// Guess the item type from file content
fn guess_item_type(content: &str) -> DocItemType {
    // In a real implementation, this would parse the file content and determine the item type
    // For now, we'll just return a placeholder
    if content.contains("struct ") {
        DocItemType::Struct
    } else if content.contains("enum ") {
        DocItemType::Enum
    } else if content.contains("trait ") {
        DocItemType::Trait
    } else if content.contains("fn ") {
        DocItemType::Function
    } else {
        DocItemType::Module
    }
}

/// Guess the item visibility from file content
fn guess_visibility(content: &str) -> DocVisibility {
    // In a real implementation, this would parse the file content and determine the visibility
    // For now, we'll just return a placeholder
    if content.contains("pub ") {
        DocVisibility::Public
    } else if content.contains("pub(crate)") {
        DocVisibility::Internal
    } else {
        DocVisibility::Private
    }
}

/// Check if an item is deprecated
fn is_deprecated(content: &str) -> bool {
    // In a real implementation, this would parse the file content and check for deprecation
    // For now, we'll just return a placeholder
    content.contains("#[deprecated")
}

/// Extract deprecation message from file content
fn extract_deprecation_message(content: &str) -> Option<String> {
    // In a real implementation, this would parse the file content and extract the deprecation message
    // For now, we'll just return a placeholder
    if content.contains("#[deprecated") {
        Some("This item is deprecated.".to_string())
    } else {
        None
    }
}

/// Extract examples from file content
fn extract_examples(content: &str) -> Vec<DocExample> {
    // In a real implementation, this would parse the file content and extract examples
    // For now, we'll just return a placeholder
    if content.contains("```") {
        vec![
            DocExample {
                title: Some("Example".to_string()),
                code: "// Example code would be extracted here".to_string(),
                output: Some("Example output".to_string()),
            }
        ]
    } else {
        Vec::new()
    }
}

/// Extract parameters from file content
fn extract_parameters(content: &str) -> Vec<DocParameter> {
    // In a real implementation, this would parse the file content and extract parameters
    // For now, we'll just return a placeholder
    if content.contains("fn ") {
        vec![
            DocParameter {
                name: "param".to_string(),
                param_type: "Type".to_string(),
                documentation: "Parameter documentation".to_string(),
                optional: false,
            }
        ]
    } else {
        Vec::new()
    }
}

/// Extract return type from file content
fn extract_return_type(content: &str) -> Option<String> {
    // In a real implementation, this would parse the file content and extract the return type
    // For now, we'll just return a placeholder
    if content.contains("-> ") {
        Some("ReturnType".to_string())
    } else {
        None
    }
}

/// Extract return documentation from file content
fn extract_return_doc(content: &str) -> Option<String> {
    // In a real implementation, this would parse the file content and extract the return documentation
    // For now, we'll just return a placeholder
    if content.contains("# Returns") {
        Some("Return documentation would be extracted here.".to_string())
    } else {
        None
    }
}

/// Initialize the API documentation generation system
pub fn init() {
    info!("Initializing API documentation generation system");
}

/// Generate API documentation
pub fn generate_api_docs(config: ApiDocsConfig, base_config: DocsGenConfig) -> io::Result<()> {
    let mut generator = ApiDocsGenerator::new(config, base_config);
    
    // Scan source directories
    generator.scan_sources()?;
    
    // Generate documentation
    generator.generate_docs()?;
    
    info!("API documentation generated successfully");
    Ok(())
}