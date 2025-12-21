use lazy_static::lazy_static;
use std::collections::HashMap;

pub struct ErrorExplanation {
    pub summary: &'static str,
    pub description: &'static str,
}

pub struct CommandInfo {
    pub package: &'static str,
    pub description: &'static str,
}

lazy_static! {
    pub static ref ERROR_INDEX: HashMap<&'static str, ErrorExplanation> = {
        let mut m = HashMap::new();
        m.insert("Undefined control sequence", ErrorExplanation {
            summary: "Unknown command",
            description: "The command you used is not defined. Check spelling or missing package.",
        });
        m.insert("Overfull \\hbox", ErrorExplanation {
            summary: "Line too wide",
            description: "The content extends beyond the margins. Try rephrasing or using a sloppypar.",
        });
        m.insert("Underfull \\hbox", ErrorExplanation {
            summary: "Line too loose",
            description: "There is too much whitespace in this line.",
        });
        m.insert("Missing $ inserted", ErrorExplanation {
            summary: "Missing math mode",
            description: "You used a math symbol (like _) outside of math mode ($...$).",
        });
        m.insert("File ended while scanning use of", ErrorExplanation {
            summary: "Unclosed command",
            description: "A command was started but the file ended before it was closed.",
        });
         m.insert("Runaway argument", ErrorExplanation {
            summary: "Unclosed argument",
            description: "An argument (usually {...}) is missing a closing brace.",
        });
        m
    };
    
    /// Maps common LaTeX commands to their required packages
    pub static ref COMMAND_PACKAGES: HashMap<&'static str, CommandInfo> = {
        let mut m = HashMap::new();
        
        // Graphics & Figures
        m.insert("\\includegraphics", CommandInfo { package: "graphicx", description: "Include external images" });
        m.insert("\\graphicspath", CommandInfo { package: "graphicx", description: "Set graphics search paths" });
        
        // Colors
        m.insert("\\textcolor", CommandInfo { package: "xcolor", description: "Colored text" });
        m.insert("\\colorbox", CommandInfo { package: "xcolor", description: "Colored box" });
        m.insert("\\definecolor", CommandInfo { package: "xcolor", description: "Define custom colors" });
        
        // Links & URLs
        m.insert("\\href", CommandInfo { package: "hyperref", description: "Clickable hyperlinks" });
        m.insert("\\url", CommandInfo { package: "hyperref", description: "Formatted URLs" });
        m.insert("\\hypersetup", CommandInfo { package: "hyperref", description: "Configure hyperlinks" });
        
        // Math (AMS packages)
        m.insert("\\text", CommandInfo { package: "amsmath", description: "Text in math mode" });
        m.insert("\\boldsymbol", CommandInfo { package: "amsmath", description: "Bold math symbols" });
        m.insert("\\mathbb", CommandInfo { package: "amssymb", description: "Blackboard bold (ℝ, ℕ, etc.)" });
        m.insert("\\mathfrak", CommandInfo { package: "amssymb", description: "Fraktur font in math" });
        m.insert("\\mathcal", CommandInfo { package: "amsmath", description: "Calligraphic math symbols" });
        m.insert("\\bm", CommandInfo { package: "bm", description: "Bold math (better than \\mathbf)" });
        
        // Tables & Arrays
        m.insert("\\toprule", CommandInfo { package: "booktabs", description: "Professional table lines" });
        m.insert("\\midrule", CommandInfo { package: "booktabs", description: "Professional table lines" });
        m.insert("\\bottomrule", CommandInfo { package: "booktabs", description: "Professional table lines" });
        m.insert("\\multirow", CommandInfo { package: "multirow", description: "Merge table rows" });
        m.insert("\\multicolumn", CommandInfo { package: "array", description: "Merge table columns" });
        
        // Formatting
        m.insert("\\setlength", CommandInfo { package: "geometry", description: "Set page dimensions" });
        m.insert("\\geometry", CommandInfo { package: "geometry", description: "Page layout configuration" });
        m.insert("\\setspace", CommandInfo { package: "setspace", description: "Line spacing control" });
        m.insert("\\doublespacing", CommandInfo { package: "setspace", description: "Double line spacing" });
        
        // Bibliography
        m.insert("\\bibliography", CommandInfo { package: "natbib or biblatex", description: "Bibliography file" });
        m.insert("\\bibliographystyle", CommandInfo { package: "natbib", description: "Bibliography style" });
        m.insert("\\citep", CommandInfo { package: "natbib", description: "Parenthetical citation" });
        m.insert("\\citet", CommandInfo { package: "natbib", description: "Textual citation" });
        m.insert("\\autocite", CommandInfo { package: "biblatex", description: "Automatic citation format" });
        
        // Code Listings
        m.insert("\\lstlisting", CommandInfo { package: "listings", description: "Code listings environment" });
        m.insert("\\lstinline", CommandInfo { package: "listings", description: "Inline code" });
        m.insert("\\mintinline", CommandInfo { package: "minted", description: "Syntax-highlighted inline code" });
        
        // TikZ & Drawing
        m.insert("\\tikz", CommandInfo { package: "tikz", description: "TikZ drawing command" });
        m.insert("\\draw", CommandInfo { package: "tikz", description: "Draw in TikZ" });
        m.insert("\\node", CommandInfo { package: "tikz", description: "Create TikZ node" });
        m.insert("\\addplot", CommandInfo { package: "pgfplots", description: "Add plot to axis (requires TikZ)" });
        
        // SI Units
        m.insert("\\si", CommandInfo { package: "siunitx", description: "SI units formatting" });
        m.insert("\\SI", CommandInfo { package: "siunitx", description: "Number with units" });
        m.insert("\\num", CommandInfo { package: "siunitx", description: "Number formatting" });
        m.insert("\\ang", CommandInfo { package: "siunitx", description: "Angle formatting" });
        
        // Subcaptions & Floats
        m.insert("\\subcaption", CommandInfo { package: "subcaption", description: "Subfigure captions" });
        m.insert("\\subfigure", CommandInfo { package: "subfig or subcaption", description: "Subfigures" });
        m.insert("\\subfloat", CommandInfo { package: "subfig", description: "Subfloat environment" });
        m.insert("\\floatplacement", CommandInfo { package: "float", description: "Control float placement" });
        
        // Font Awesome & Icons
        m.insert("\\faGithub", CommandInfo { package: "fontawesome5", description: "Font Awesome icons" });
        m.insert("\\faEnvelope", CommandInfo { package: "fontawesome5", description: "Font Awesome icons" });
        m.insert("\\faLinkedin", CommandInfo { package: "fontawesome5", description: "Font Awesome icons" });
        
        // Enhanced Lists
        m.insert("\\setlist", CommandInfo { package: "enumitem", description: "Customize list formatting" });
        m.insert("\\setitemize", CommandInfo { package: "enumitem", description: "Customize itemize lists" });
        
        // Cross-References
        m.insert("\\cref", CommandInfo { package: "cleveref", description: "Smart cross-reference (auto-adds type)" });
        m.insert("\\Cref", CommandInfo { package: "cleveref", description: "Capitalized smart cross-reference" });
        m.insert("\\crefrange", CommandInfo { package: "cleveref", description: "Reference range" });
        
        // Quotations
        m.insert("\\enquote", CommandInfo { package: "csquotes", description: "Context-sensitive quotations" });
        m.insert("\\blockquote", CommandInfo { package: "csquotes", description: "Block quotation environment" });
        
        // Advanced Tables
        m.insert("\\makecell", CommandInfo { package: "makecell", description: "Multi-line cells in tables" });
        m.insert("\\thead", CommandInfo { package: "makecell", description: "Table header formatting" });
        m.insert("\\tabularx", CommandInfo { package: "tabularx", description: "Auto-width table columns" });
        m.insert("\\longtable", CommandInfo { package: "longtable", description: "Multi-page tables" });
        m.insert("\\hhline", CommandInfo { package: "hhline", description: "Custom horizontal/vertical table lines" });
        
        // Theorems & Proofs
        m.insert("\\newtheorem", CommandInfo { package: "amsthm", description: "Define theorem environments" });
        m.insert("\\theoremstyle", CommandInfo { package: "amsthm", description: "Set theorem style" });
        m.insert("\\proof", CommandInfo { package: "amsthm", description: "Proof environment" });
        m.insert("\\qedhere", CommandInfo { package: "amsthm", description: "Position QED symbol" });
        
        // Algorithms & Pseudocode
        m.insert("\\algorithm", CommandInfo { package: "algorithm or algorithm2e", description: "Algorithm environment" });
        m.insert("\\algorithmic", CommandInfo { package: "algorithmicx", description: "Algorithmic pseudocode" });
        m.insert("\\If", CommandInfo { package: "algorithmicx", description: "If statement in algorithm" });
        m.insert("\\While", CommandInfo { package: "algorithmicx", description: "While loop in algorithm" });
        
        // Chemical Formulas
        m.insert("\\ce", CommandInfo { package: "mhchem", description: "Chemical equations and formulas" });
        
        // Appendices
        m.insert("\\appendixpage", CommandInfo { package: "appendix", description: "Appendix title page" });
        m.insert("\\appendixname", CommandInfo { package: "appendix", description: "Customize appendix name" });
        
        // Headers & Footers
        m.insert("\\fancyhead", CommandInfo { package: "fancyhdr", description: "Custom page headers" });
        m.insert("\\fancyfoot", CommandInfo { package: "fancyhdr", description: "Custom page footers" });
        m.insert("\\fancyhf", CommandInfo { package: "fancyhdr", description: "Set header and footer" });
        
        // Typography & Micro-typography
        m.insert("\\textls", CommandInfo { package: "microtype", description: "Letter spacing adjustment" });
        
        // Dates
        m.insert("\\today", CommandInfo { package: "built-in", description: "Current date (always available)" });
        m.insert("\\formatdate", CommandInfo { package: "datetime", description: "Format dates" });
        
        m
    };
}

pub fn explain(message: &str) -> Option<&'static ErrorExplanation> {
    for (key, explanation) in ERROR_INDEX.iter() {
        if message.contains(key) {
            return Some(explanation);
        }
    }
    None
}

/// Attempts to extract the undefined command from an error message
/// Returns the command name if found (including leading backslash)
pub fn extract_undefined_command(message: &str) -> Option<String> {
    // Tectonic format: "Undefined control sequence" followed by the command on next line
    // But in our message it's just one line string
    // We need to check if message contains "\commandname" pattern
    
    // Pattern: Look for backslash followed by letters
    let words: Vec<&str> = message.split_whitespace().collect();
    for word in words {
        if word.starts_with('\\') {
            // Extract command name (stop at non-alphanumeric)
            let cmd: String = word.chars()
                .take_while(|c| c.is_alphanumeric() || *c == '\\')
                .collect();
            if cmd.len() > 1 { // At least "\x"
                return Some(cmd);
            }
        }
    }
    None
}

/// Provides a helpful suggestion for an undefined command
pub fn suggest_package(command: &str) -> Option<String> {
    if let Some(info) = COMMAND_PACKAGES.get(command) {
        Some(format!(
            "💡 Add `\\usepackage{{{}}}` to use `{}` ({})",
            info.package,
            command,
            info.description
        ))
    } else {
        None
    }
}
