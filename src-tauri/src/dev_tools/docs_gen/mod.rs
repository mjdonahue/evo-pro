//! Developer Documentation Generation Tools
//!
//! This module provides tools for automatically generating developer documentation
//! from the codebase. It includes utilities for generating API documentation,
//! extracting code examples, and creating visual documentation.

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use serde::{Serialize, Deserialize};
use tracing::{debug, info, warn, error};

pub mod api;
pub mod interactive;

/// Documentation generation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocsGenConfig {
    /// Whether documentation generation is enabled
    pub enabled: bool,
    /// Output directory for generated documentation
    pub output_dir: String,
    /// Source directories to scan for documentation
    pub source_dirs: Vec<String>,
    /// Whether to include private items
    pub include_private: bool,
    /// Whether to include internal items
    pub include_internal: bool,
    /// Whether to generate API documentation
    pub generate_api_docs: bool,
    /// Whether to generate code examples
    pub generate_examples: bool,
    /// Whether to generate visual documentation
    pub generate_visual_docs: bool,
    /// Template directory for documentation generation
    pub template_dir: Option<String>,
    /// Additional options
    pub options: HashMap<String, String>,
}

impl Default for DocsGenConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            output_dir: "docs/generated".to_string(),
            source_dirs: vec!["src".to_string(), "src-tauri/src".to_string()],
            include_private: false,
            include_internal: false,
            generate_api_docs: true,
            generate_examples: true,
            generate_visual_docs: true,
            template_dir: None,
            options: HashMap::new(),
        }
    }
}

/// Documentation item type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocItemType {
    /// Module
    Module,
    /// Struct
    Struct,
    /// Enum
    Enum,
    /// Trait
    Trait,
    /// Function
    Function,
    /// Method
    Method,
    /// Constant
    Constant,
    /// Type alias
    TypeAlias,
    /// Macro
    Macro,
}

/// Documentation item visibility
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocVisibility {
    /// Public
    Public,
    /// Private
    Private,
    /// Internal (pub(crate))
    Internal,
}

/// Documentation item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocItem {
    /// Item name
    pub name: String,
    /// Item type
    pub item_type: DocItemType,
    /// Item visibility
    pub visibility: DocVisibility,
    /// Item path
    pub path: String,
    /// Item documentation
    pub documentation: String,
    /// Source file
    pub source_file: String,
    /// Line number
    pub line: usize,
    /// Whether the item is deprecated
    pub deprecated: bool,
    /// Deprecation message
    pub deprecation_message: Option<String>,
    /// Examples
    pub examples: Vec<DocExample>,
    /// Parameters (for functions and methods)
    pub parameters: Vec<DocParameter>,
    /// Return type (for functions and methods)
    pub return_type: Option<String>,
    /// Return documentation (for functions and methods)
    pub return_documentation: Option<String>,
    /// Associated items (for structs, enums, and traits)
    pub associated_items: Vec<DocItem>,
    /// Parent item
    pub parent: Option<Box<DocItem>>,
}

/// Documentation example
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocExample {
    /// Example title
    pub title: Option<String>,
    /// Example code
    pub code: String,
    /// Example output
    pub output: Option<String>,
}

/// Documentation parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocParameter {
    /// Parameter name
    pub name: String,
    /// Parameter type
    pub param_type: String,
    /// Parameter documentation
    pub documentation: String,
    /// Whether the parameter is optional
    pub optional: bool,
}

/// Documentation generator
#[derive(Debug)]
pub struct DocsGenerator {
    /// Configuration
    pub config: DocsGenConfig,
    /// Documentation items
    pub items: Vec<DocItem>,
    /// Documentation templates
    pub templates: HashMap<String, String>,
}

impl DocsGenerator {
    /// Create a new documentation generator
    pub fn new(config: DocsGenConfig) -> Self {
        Self {
            config,
            items: Vec::new(),
            templates: HashMap::new(),
        }
    }

    /// Load templates from the template directory
    pub fn load_templates(&mut self) -> io::Result<()> {
        if let Some(template_dir) = &self.config.template_dir {
            let template_path = Path::new(template_dir);
            if template_path.exists() && template_path.is_dir() {
                for entry in fs::read_dir(template_path)? {
                    let entry = entry?;
                    let path = entry.path();
                    if path.is_file() && path.extension().map_or(false, |ext| ext == "hbs") {
                        let mut file = File::open(&path)?;
                        let mut contents = String::new();
                        file.read_to_string(&mut contents)?;

                        if let Some(file_name) = path.file_stem() {
                            if let Some(file_name_str) = file_name.to_str() {
                                self.templates.insert(file_name_str.to_string(), contents);
                            }
                        }
                    }
                }
            }
        }

        // Add default templates if not loaded
        if !self.templates.contains_key("module") {
            self.templates.insert("module".to_string(), include_str!("templates/module.hbs").to_string());
        }
        if !self.templates.contains_key("struct") {
            self.templates.insert("struct".to_string(), include_str!("templates/struct.hbs").to_string());
        }
        if !self.templates.contains_key("enum") {
            self.templates.insert("enum".to_string(), include_str!("templates/enum.hbs").to_string());
        }
        if !self.templates.contains_key("trait") {
            self.templates.insert("trait".to_string(), include_str!("templates/trait.hbs").to_string());
        }
        if !self.templates.contains_key("function") {
            self.templates.insert("function".to_string(), include_str!("templates/function.hbs").to_string());
        }
        if !self.templates.contains_key("index") {
            self.templates.insert("index".to_string(), include_str!("templates/index.hbs").to_string());
        }

        Ok(())
    }

    /// Scan source directories for documentation
    pub fn scan_sources(&mut self) -> io::Result<()> {
        for source_dir in &self.config.source_dirs {
            let source_path = Path::new(source_dir);
            if source_path.exists() && source_path.is_dir() {
                self.scan_directory(source_path)?;
            } else {
                warn!("Source directory does not exist: {}", source_dir);
            }
        }

        Ok(())
    }

    /// Scan a directory for documentation
    fn scan_directory(&mut self, dir: &Path) -> io::Result<()> {
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

    /// Scan a file for documentation
    fn scan_file(&mut self, file: &Path) -> io::Result<()> {
        let mut file_content = String::new();
        let mut file_handle = File::open(file)?;
        file_handle.read_to_string(&mut file_content)?;

        // In a real implementation, this would parse the Rust file and extract documentation
        // For now, we'll just create a simple representation based on the file name

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
                        documentation: extract_module_doc(&file_content),
                        source_file: file.to_string_lossy().to_string(),
                        line: 1,
                        deprecated: false,
                        deprecation_message: None,
                        examples: extract_examples(&file_content),
                        parameters: Vec::new(),
                        return_type: None,
                        return_documentation: None,
                        associated_items: Vec::new(),
                        parent: None,
                    };

                    self.items.push(doc_item);
                } else if file_name_str.ends_with(".rs") {
                    // This is a regular Rust file
                    let item_name = file_name_str.trim_end_matches(".rs");

                    let doc_item = DocItem {
                        name: item_name.to_string(),
                        item_type: guess_item_type(&file_content),
                        visibility: guess_visibility(&file_content),
                        path: format!("crate::{}", item_name),
                        documentation: extract_item_doc(&file_content),
                        source_file: file.to_string_lossy().to_string(),
                        line: 1,
                        deprecated: is_deprecated(&file_content),
                        deprecation_message: extract_deprecation_message(&file_content),
                        examples: extract_examples(&file_content),
                        parameters: extract_parameters(&file_content),
                        return_type: extract_return_type(&file_content),
                        return_documentation: extract_return_doc(&file_content),
                        associated_items: Vec::new(),
                        parent: None,
                    };

                    self.items.push(doc_item);
                }
            }
        }

        Ok(())
    }

    /// Generate documentation
    pub fn generate_docs(&self) -> io::Result<()> {
        // Create output directory if it doesn't exist
        let output_dir = Path::new(&self.config.output_dir);
        fs::create_dir_all(output_dir)?;

        // Generate API documentation if enabled
        if self.config.generate_api_docs {
            self.generate_api_docs(output_dir)?;
        }

        // Generate code examples if enabled
        if self.config.generate_examples {
            self.generate_examples(output_dir)?;
        }

        // Generate visual documentation if enabled
        if self.config.generate_visual_docs {
            self.generate_visual_docs(output_dir)?;
        }

        Ok(())
    }

    /// Generate API documentation
    fn generate_api_docs(&self, output_dir: &Path) -> io::Result<()> {
        let api_dir = output_dir.join("api");
        fs::create_dir_all(&api_dir)?;

        // Generate index file
        let index_path = api_dir.join("index.html");
        let mut index_file = File::create(index_path)?;

        // In a real implementation, this would use a template engine like Handlebars
        // For now, we'll just generate a simple HTML file

        writeln!(index_file, "<!DOCTYPE html>")?;
        writeln!(index_file, "<html>")?;
        writeln!(index_file, "<head>")?;
        writeln!(index_file, "    <title>API Documentation</title>")?;
        writeln!(index_file, "    <style>")?;
        writeln!(index_file, "        body {{ font-family: Arial, sans-serif; margin: 0; padding: 20px; }}")?;
        writeln!(index_file, "        h1 {{ color: #333; }}")?;
        writeln!(index_file, "        .item {{ margin-bottom: 10px; }}")?;
        writeln!(index_file, "        .item a {{ text-decoration: none; color: #0077cc; }}")?;
        writeln!(index_file, "        .item a:hover {{ text-decoration: underline; }}")?;
        writeln!(index_file, "        .item-type {{ color: #666; font-size: 0.8em; }}")?;
        writeln!(index_file, "    </style>")?;
        writeln!(index_file, "</head>")?;
        writeln!(index_file, "<body>")?;
        writeln!(index_file, "    <h1>API Documentation</h1>")?;

        // Group items by type
        let mut modules = Vec::new();
        let mut structs = Vec::new();
        let mut enums = Vec::new();
        let mut traits = Vec::new();
        let mut functions = Vec::new();

        for item in &self.items {
            match item.item_type {
                DocItemType::Module => modules.push(item),
                DocItemType::Struct => structs.push(item),
                DocItemType::Enum => enums.push(item),
                DocItemType::Trait => traits.push(item),
                DocItemType::Function => functions.push(item),
                _ => {}
            }
        }

        // Write modules
        if !modules.is_empty() {
            writeln!(index_file, "    <h2>Modules</h2>")?;
            for module in modules {
                writeln!(index_file, "    <div class=\"item\">")?;
                writeln!(index_file, "        <a href=\"{}.html\">{}</a>", module.name, module.name)?;
                writeln!(index_file, "        <span class=\"item-type\">module</span>")?;
                writeln!(index_file, "    </div>")?;

                // Generate module page
                self.generate_module_page(module, &api_dir)?;
            }
        }

        // Write structs
        if !structs.is_empty() {
            writeln!(index_file, "    <h2>Structs</h2>")?;
            for struct_item in structs {
                writeln!(index_file, "    <div class=\"item\">")?;
                writeln!(index_file, "        <a href=\"{}.html\">{}</a>", struct_item.name, struct_item.name)?;
                writeln!(index_file, "        <span class=\"item-type\">struct</span>")?;
                writeln!(index_file, "    </div>")?;

                // Generate struct page
                self.generate_struct_page(struct_item, &api_dir)?;
            }
        }

        // Write enums
        if !enums.is_empty() {
            writeln!(index_file, "    <h2>Enums</h2>")?;
            for enum_item in enums {
                writeln!(index_file, "    <div class=\"item\">")?;
                writeln!(index_file, "        <a href=\"{}.html\">{}</a>", enum_item.name, enum_item.name)?;
                writeln!(index_file, "        <span class=\"item-type\">enum</span>")?;
                writeln!(index_file, "    </div>")?;

                // Generate enum page
                self.generate_enum_page(enum_item, &api_dir)?;
            }
        }

        // Write traits
        if !traits.is_empty() {
            writeln!(index_file, "    <h2>Traits</h2>")?;
            for trait_item in traits {
                writeln!(index_file, "    <div class=\"item\">")?;
                writeln!(index_file, "        <a href=\"{}.html\">{}</a>", trait_item.name, trait_item.name)?;
                writeln!(index_file, "        <span class=\"item-type\">trait</span>")?;
                writeln!(index_file, "    </div>")?;

                // Generate trait page
                self.generate_trait_page(trait_item, &api_dir)?;
            }
        }

        // Write functions
        if !functions.is_empty() {
            writeln!(index_file, "    <h2>Functions</h2>")?;
            for function in functions {
                writeln!(index_file, "    <div class=\"item\">")?;
                writeln!(index_file, "        <a href=\"{}.html\">{}</a>", function.name, function.name)?;
                writeln!(index_file, "        <span class=\"item-type\">function</span>")?;
                writeln!(index_file, "    </div>")?;

                // Generate function page
                self.generate_function_page(function, &api_dir)?;
            }
        }

        writeln!(index_file, "</body>")?;
        writeln!(index_file, "</html>")?;

        Ok(())
    }

    /// Generate a module documentation page
    fn generate_module_page(&self, module: &DocItem, output_dir: &Path) -> io::Result<()> {
        let page_path = output_dir.join(format!("{}.html", module.name));
        let mut page_file = File::create(page_path)?;

        writeln!(page_file, "<!DOCTYPE html>")?;
        writeln!(page_file, "<html>")?;
        writeln!(page_file, "<head>")?;
        writeln!(page_file, "    <title>Module {}</title>", module.name)?;
        writeln!(page_file, "    <style>")?;
        writeln!(page_file, "        body {{ font-family: Arial, sans-serif; margin: 0; padding: 20px; }}")?;
        writeln!(page_file, "        h1 {{ color: #333; }}")?;
        writeln!(page_file, "        .item {{ margin-bottom: 10px; }}")?;
        writeln!(page_file, "        .item a {{ text-decoration: none; color: #0077cc; }}")?;
        writeln!(page_file, "        .item a:hover {{ text-decoration: underline; }}")?;
        writeln!(page_file, "        .item-type {{ color: #666; font-size: 0.8em; }}")?;
        writeln!(page_file, "        .documentation {{ margin-top: 20px; }}")?;
        writeln!(page_file, "        .example {{ background-color: #f5f5f5; padding: 10px; margin-top: 10px; }}")?;
        writeln!(page_file, "        .example pre {{ margin: 0; }}")?;
        writeln!(page_file, "    </style>")?;
        writeln!(page_file, "</head>")?;
        writeln!(page_file, "<body>")?;
        writeln!(page_file, "    <h1>Module {}</h1>", module.name)?;

        // Write documentation
        writeln!(page_file, "    <div class=\"documentation\">")?;
        writeln!(page_file, "        <p>{}</p>", module.documentation)?;
        writeln!(page_file, "    </div>")?;

        // Write examples
        if !module.examples.is_empty() {
            writeln!(page_file, "    <h2>Examples</h2>")?;
            for example in &module.examples {
                writeln!(page_file, "    <div class=\"example\">")?;
                if let Some(title) = &example.title {
                    writeln!(page_file, "        <h3>{}</h3>", title)?;
                }
                writeln!(page_file, "        <pre>{}</pre>", example.code)?;
                if let Some(output) = &example.output {
                    writeln!(page_file, "        <h4>Output:</h4>")?;
                    writeln!(page_file, "        <pre>{}</pre>", output)?;
                }
                writeln!(page_file, "    </div>")?;
            }
        }

        // Write associated items
        if !module.associated_items.is_empty() {
            writeln!(page_file, "    <h2>Items</h2>")?;

            // Group items by type
            let mut structs = Vec::new();
            let mut enums = Vec::new();
            let mut traits = Vec::new();
            let mut functions = Vec::new();

            for item in &module.associated_items {
                match item.item_type {
                    DocItemType::Struct => structs.push(item),
                    DocItemType::Enum => enums.push(item),
                    DocItemType::Trait => traits.push(item),
                    DocItemType::Function => functions.push(item),
                    _ => {}
                }
            }

            // Write structs
            if !structs.is_empty() {
                writeln!(page_file, "    <h3>Structs</h3>")?;
                for struct_item in structs {
                    writeln!(page_file, "    <div class=\"item\">")?;
                    writeln!(page_file, "        <a href=\"{}.html\">{}</a>", struct_item.name, struct_item.name)?;
                    writeln!(page_file, "        <span class=\"item-type\">struct</span>")?;
                    writeln!(page_file, "    </div>")?;
                }
            }

            // Write enums
            if !enums.is_empty() {
                writeln!(page_file, "    <h3>Enums</h3>")?;
                for enum_item in enums {
                    writeln!(page_file, "    <div class=\"item\">")?;
                    writeln!(page_file, "        <a href=\"{}.html\">{}</a>", enum_item.name, enum_item.name)?;
                    writeln!(page_file, "        <span class=\"item-type\">enum</span>")?;
                    writeln!(page_file, "    </div>")?;
                }
            }

            // Write traits
            if !traits.is_empty() {
                writeln!(page_file, "    <h3>Traits</h3>")?;
                for trait_item in traits {
                    writeln!(page_file, "    <div class=\"item\">")?;
                    writeln!(page_file, "        <a href=\"{}.html\">{}</a>", trait_item.name, trait_item.name)?;
                    writeln!(page_file, "        <span class=\"item-type\">trait</span>")?;
                    writeln!(page_file, "    </div>")?;
                }
            }

            // Write functions
            if !functions.is_empty() {
                writeln!(page_file, "    <h3>Functions</h3>")?;
                for function in functions {
                    writeln!(page_file, "    <div class=\"item\">")?;
                    writeln!(page_file, "        <a href=\"{}.html\">{}</a>", function.name, function.name)?;
                    writeln!(page_file, "        <span class=\"item-type\">function</span>")?;
                    writeln!(page_file, "    </div>")?;
                }
            }
        }

        writeln!(page_file, "    <p><a href=\"index.html\">Back to index</a></p>")?;
        writeln!(page_file, "</body>")?;
        writeln!(page_file, "</html>")?;

        Ok(())
    }

    /// Generate a struct documentation page
    fn generate_struct_page(&self, struct_item: &DocItem, output_dir: &Path) -> io::Result<()> {
        let page_path = output_dir.join(format!("{}.html", struct_item.name));
        let mut page_file = File::create(page_path)?;

        writeln!(page_file, "<!DOCTYPE html>")?;
        writeln!(page_file, "<html>")?;
        writeln!(page_file, "<head>")?;
        writeln!(page_file, "    <title>Struct {}</title>", struct_item.name)?;
        writeln!(page_file, "    <style>")?;
        writeln!(page_file, "        body {{ font-family: Arial, sans-serif; margin: 0; padding: 20px; }}")?;
        writeln!(page_file, "        h1 {{ color: #333; }}")?;
        writeln!(page_file, "        .item {{ margin-bottom: 10px; }}")?;
        writeln!(page_file, "        .item a {{ text-decoration: none; color: #0077cc; }}")?;
        writeln!(page_file, "        .item a:hover {{ text-decoration: underline; }}")?;
        writeln!(page_file, "        .item-type {{ color: #666; font-size: 0.8em; }}")?;
        writeln!(page_file, "        .documentation {{ margin-top: 20px; }}")?;
        writeln!(page_file, "        .example {{ background-color: #f5f5f5; padding: 10px; margin-top: 10px; }}")?;
        writeln!(page_file, "        .example pre {{ margin: 0; }}")?;
        writeln!(page_file, "    </style>")?;
        writeln!(page_file, "</head>")?;
        writeln!(page_file, "<body>")?;
        writeln!(page_file, "    <h1>Struct {}</h1>", struct_item.name)?;

        // Write documentation
        writeln!(page_file, "    <div class=\"documentation\">")?;
        writeln!(page_file, "        <p>{}</p>", struct_item.documentation)?;
        writeln!(page_file, "    </div>")?;

        // Write examples
        if !struct_item.examples.is_empty() {
            writeln!(page_file, "    <h2>Examples</h2>")?;
            for example in &struct_item.examples {
                writeln!(page_file, "    <div class=\"example\">")?;
                if let Some(title) = &example.title {
                    writeln!(page_file, "        <h3>{}</h3>", title)?;
                }
                writeln!(page_file, "        <pre>{}</pre>", example.code)?;
                if let Some(output) = &example.output {
                    writeln!(page_file, "        <h4>Output:</h4>")?;
                    writeln!(page_file, "        <pre>{}</pre>", output)?;
                }
                writeln!(page_file, "    </div>")?;
            }
        }

        // Write methods
        let methods: Vec<&DocItem> = struct_item.associated_items.iter()
            .filter(|item| item.item_type == DocItemType::Method)
            .collect();

        if !methods.is_empty() {
            writeln!(page_file, "    <h2>Methods</h2>")?;
            for method in methods {
                writeln!(page_file, "    <div class=\"method\">")?;
                writeln!(page_file, "        <h3>{}</h3>", method.name)?;
                writeln!(page_file, "        <div class=\"documentation\">")?;
                writeln!(page_file, "            <p>{}</p>", method.documentation)?;
                writeln!(page_file, "        </div>")?;

                // Write parameters
                if !method.parameters.is_empty() {
                    writeln!(page_file, "        <h4>Parameters</h4>")?;
                    writeln!(page_file, "        <ul>")?;
                    for param in &method.parameters {
                        writeln!(page_file, "            <li>")?;
                        writeln!(page_file, "                <code>{}: {}</code>", param.name, param.param_type)?;
                        writeln!(page_file, "                <p>{}</p>", param.documentation)?;
                        writeln!(page_file, "            </li>")?;
                    }
                    writeln!(page_file, "        </ul>")?;
                }

                // Write return type and documentation
                if let Some(return_type) = &method.return_type {
                    writeln!(page_file, "        <h4>Returns</h4>")?;
                    writeln!(page_file, "        <p><code>{}</code></p>", return_type)?;
                    if let Some(return_doc) = &method.return_documentation {
                        writeln!(page_file, "        <p>{}</p>", return_doc)?;
                    }
                }

                writeln!(page_file, "    </div>")?;
            }
        }

        writeln!(page_file, "    <p><a href=\"index.html\">Back to index</a></p>")?;
        writeln!(page_file, "</body>")?;
        writeln!(page_file, "</html>")?;

        Ok(())
    }

    /// Generate an enum documentation page
    fn generate_enum_page(&self, enum_item: &DocItem, output_dir: &Path) -> io::Result<()> {
        // Similar to generate_struct_page, but for enums
        let page_path = output_dir.join(format!("{}.html", enum_item.name));
        let mut page_file = File::create(page_path)?;

        writeln!(page_file, "<!DOCTYPE html>")?;
        writeln!(page_file, "<html>")?;
        writeln!(page_file, "<head>")?;
        writeln!(page_file, "    <title>Enum {}</title>", enum_item.name)?;
        writeln!(page_file, "    <style>")?;
        writeln!(page_file, "        body {{ font-family: Arial, sans-serif; margin: 0; padding: 20px; }}")?;
        writeln!(page_file, "        h1 {{ color: #333; }}")?;
        writeln!(page_file, "        .documentation {{ margin-top: 20px; }}")?;
        writeln!(page_file, "    </style>")?;
        writeln!(page_file, "</head>")?;
        writeln!(page_file, "<body>")?;
        writeln!(page_file, "    <h1>Enum {}</h1>", enum_item.name)?;
        writeln!(page_file, "    <div class=\"documentation\">")?;
        writeln!(page_file, "        <p>{}</p>", enum_item.documentation)?;
        writeln!(page_file, "    </div>")?;
        writeln!(page_file, "    <p><a href=\"index.html\">Back to index</a></p>")?;
        writeln!(page_file, "</body>")?;
        writeln!(page_file, "</html>")?;

        Ok(())
    }

    /// Generate a trait documentation page
    fn generate_trait_page(&self, trait_item: &DocItem, output_dir: &Path) -> io::Result<()> {
        // Similar to generate_struct_page, but for traits
        let page_path = output_dir.join(format!("{}.html", trait_item.name));
        let mut page_file = File::create(page_path)?;

        writeln!(page_file, "<!DOCTYPE html>")?;
        writeln!(page_file, "<html>")?;
        writeln!(page_file, "<head>")?;
        writeln!(page_file, "    <title>Trait {}</title>", trait_item.name)?;
        writeln!(page_file, "    <style>")?;
        writeln!(page_file, "        body {{ font-family: Arial, sans-serif; margin: 0; padding: 20px; }}")?;
        writeln!(page_file, "        h1 {{ color: #333; }}")?;
        writeln!(page_file, "        .documentation {{ margin-top: 20px; }}")?;
        writeln!(page_file, "    </style>")?;
        writeln!(page_file, "</head>")?;
        writeln!(page_file, "<body>")?;
        writeln!(page_file, "    <h1>Trait {}</h1>", trait_item.name)?;
        writeln!(page_file, "    <div class=\"documentation\">")?;
        writeln!(page_file, "        <p>{}</p>", trait_item.documentation)?;
        writeln!(page_file, "    </div>")?;
        writeln!(page_file, "    <p><a href=\"index.html\">Back to index</a></p>")?;
        writeln!(page_file, "</body>")?;
        writeln!(page_file, "</html>")?;

        Ok(())
    }

    /// Generate a function documentation page
    fn generate_function_page(&self, function: &DocItem, output_dir: &Path) -> io::Result<()> {
        // Similar to generate_struct_page, but for functions
        let page_path = output_dir.join(format!("{}.html", function.name));
        let mut page_file = File::create(page_path)?;

        writeln!(page_file, "<!DOCTYPE html>")?;
        writeln!(page_file, "<html>")?;
        writeln!(page_file, "<head>")?;
        writeln!(page_file, "    <title>Function {}</title>", function.name)?;
        writeln!(page_file, "    <style>")?;
        writeln!(page_file, "        body {{ font-family: Arial, sans-serif; margin: 0; padding: 20px; }}")?;
        writeln!(page_file, "        h1 {{ color: #333; }}")?;
        writeln!(page_file, "        .documentation {{ margin-top: 20px; }}")?;
        writeln!(page_file, "    </style>")?;
        writeln!(page_file, "</head>")?;
        writeln!(page_file, "<body>")?;
        writeln!(page_file, "    <h1>Function {}</h1>", function.name)?;
        writeln!(page_file, "    <div class=\"documentation\">")?;
        writeln!(page_file, "        <p>{}</p>", function.documentation)?;
        writeln!(page_file, "    </div>")?;
        writeln!(page_file, "    <p><a href=\"index.html\">Back to index</a></p>")?;
        writeln!(page_file, "</body>")?;
        writeln!(page_file, "</html>")?;

        Ok(())
    }

    /// Generate code examples
    fn generate_examples(&self, output_dir: &Path) -> io::Result<()> {
        let examples_dir = output_dir.join("examples");
        fs::create_dir_all(&examples_dir)?;

        // Generate index file
        let index_path = examples_dir.join("index.html");
        let mut index_file = File::create(index_path)?;

        writeln!(index_file, "<!DOCTYPE html>")?;
        writeln!(index_file, "<html>")?;
        writeln!(index_file, "<head>")?;
        writeln!(index_file, "    <title>Code Examples</title>")?;
        writeln!(index_file, "    <style>")?;
        writeln!(index_file, "        body {{ font-family: Arial, sans-serif; margin: 0; padding: 20px; }}")?;
        writeln!(index_file, "        h1 {{ color: #333; }}")?;
        writeln!(index_file, "        .example {{ margin-bottom: 20px; }}")?;
        writeln!(index_file, "        .example h3 {{ margin-bottom: 5px; }}")?;
        writeln!(index_file, "        .example pre {{ background-color: #f5f5f5; padding: 10px; margin-top: 5px; }}")?;
        writeln!(index_file, "    </style>")?;
        writeln!(index_file, "</head>")?;
        writeln!(index_file, "<body>")?;
        writeln!(index_file, "    <h1>Code Examples</h1>")?;

        // Collect all examples
        let mut all_examples = Vec::new();

        for item in &self.items {
            for example in &item.examples {
                all_examples.push((item, example));
            }
        }

        // Write examples
        if all_examples.is_empty() {
            writeln!(index_file, "    <p>No examples found.</p>")?;
        } else {
            for (item, example) in all_examples {
                writeln!(index_file, "    <div class=\"example\">")?;
                if let Some(title) = &example.title {
                    writeln!(index_file, "        <h3>{}</h3>", title)?;
                } else {
                    writeln!(index_file, "        <h3>Example for {}</h3>", item.name)?;
                }
                writeln!(index_file, "        <p>From: {}</p>", item.path)?;
                writeln!(index_file, "        <pre>{}</pre>", example.code)?;
                if let Some(output) = &example.output {
                    writeln!(index_file, "        <h4>Output:</h4>")?;
                    writeln!(index_file, "        <pre>{}</pre>", output)?;
                }
                writeln!(index_file, "    </div>")?;
            }
        }

        writeln!(index_file, "</body>")?;
        writeln!(index_file, "</html>")?;

        Ok(())
    }

    /// Generate visual documentation
    fn generate_visual_docs(&self, output_dir: &Path) -> io::Result<()> {
        let visual_dir = output_dir.join("visual");
        fs::create_dir_all(&visual_dir)?;

        // Generate index file
        let index_path = visual_dir.join("index.html");
        let mut index_file = File::create(index_path)?;

        writeln!(index_file, "<!DOCTYPE html>")?;
        writeln!(index_file, "<html>")?;
        writeln!(index_file, "<head>")?;
        writeln!(index_file, "    <title>Visual Documentation</title>")?;
        writeln!(index_file, "    <style>")?;
        writeln!(index_file, "        body {{ font-family: Arial, sans-serif; margin: 0; padding: 20px; }}")?;
        writeln!(index_file, "        h1 {{ color: #333; }}")?;
        writeln!(index_file, "    </style>")?;
        writeln!(index_file, "</head>")?;
        writeln!(index_file, "<body>")?;
        writeln!(index_file, "    <h1>Visual Documentation</h1>")?;

        // In a real implementation, this would generate diagrams, charts, etc.
        // For now, we'll just generate a placeholder

        writeln!(index_file, "    <p>Visual documentation would be generated here.</p>")?;
        writeln!(index_file, "    <p>This could include:</p>")?;
        writeln!(index_file, "    <ul>")?;
        writeln!(index_file, "        <li>Class diagrams</li>")?;
        writeln!(index_file, "        <li>Sequence diagrams</li>")?;
        writeln!(index_file, "        <li>Component diagrams</li>")?;
        writeln!(index_file, "        <li>Dependency graphs</li>")?;
        writeln!(index_file, "    </ul>")?;

        writeln!(index_file, "</body>")?;
        writeln!(index_file, "</html>")?;

        Ok(())
    }
}

/// Global documentation generator
lazy_static::lazy_static! {
    static ref DOCS_GENERATOR: Arc<Mutex<DocsGenerator>> = Arc::new(Mutex::new(
        DocsGenerator::new(DocsGenConfig::default())
    ));
}

/// Get the global documentation generator
pub fn get_docs_generator() -> Arc<Mutex<DocsGenerator>> {
    DOCS_GENERATOR.clone()
}

/// Configure documentation generation
pub fn configure(config: DocsGenConfig) {
    let mut generator = DOCS_GENERATOR.lock().unwrap();
    *generator = DocsGenerator::new(config);
}

/// Generate documentation
pub fn generate_docs() -> io::Result<()> {
    let mut generator = DOCS_GENERATOR.lock().unwrap();

    // Check if documentation generation is enabled
    if !generator.config.enabled {
        info!("Documentation generation is disabled");
        return Ok(());
    }

    // Load templates
    generator.load_templates()?;

    // Scan source directories
    generator.scan_sources()?;

    // Generate documentation
    generator.generate_docs()?;

    info!("Documentation generated successfully");
    Ok(())
}

/// Initialize the documentation generation system
pub fn init() {
    info!("Initializing documentation generation system");
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
