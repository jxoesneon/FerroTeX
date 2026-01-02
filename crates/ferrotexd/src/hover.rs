use ferrotex_syntax::{SyntaxKind, SyntaxNode, TextSize};
use rowan::TokenAtOffset;
use tower_lsp::lsp_types::{Hover, HoverContents, MarkupContent, MarkupKind};

/// Computes hover information for the given cursor position.
/// 
/// Supports:
/// - Citations (`\cite{key}`) → Shows bibliography details
/// - Math environments (`\begin{equation}`) → Shows helpful tip
/// - Common commands → Shows documentation
/// - Packages → Shows package info
pub fn find_hover(root: &SyntaxNode, offset: TextSize, workspace: &crate::workspace::Workspace) -> Option<Hover> {
    let token = match root.token_at_offset(offset) {
        TokenAtOffset::None => return None,
        TokenAtOffset::Single(t) => t,
        TokenAtOffset::Between(l, r) => {
            if l.kind() != SyntaxKind::Whitespace {
                l
            } else {
                r
            }
        }
    };

    // Check parent nodes for context
    let mut current = token.parent()?;
    
    // First check: are we directly on a command?
    if current.kind() == SyntaxKind::Command {
        return handle_command_hover(&current.to_string());
    }
    
    // Check for citation (can be inside command groups)
    while current.kind() != SyntaxKind::Root {
        match current.kind() {
            SyntaxKind::Citation => {
                return handle_citation_hover(&current, workspace);
            }
            SyntaxKind::Environment => {
                // strict check: only show environment hover if we're on the \begin or \end token
                let token_text = token.text();
                // Check if we are hovering exactly on \begin, \end, begin, end, or the environment name inside braces
                if token_text == "\\begin" || token_text == "\\end" 
                    || token_text == "begin" || token_text == "end" {
                    return handle_environment_hover(&current);
                }
            }
            _ => {}
        }
        current = current.parent()?;
    }
    
    // Fallback for flat parser trees (where parent is Root):
    // Check if the token text looks like a command
    if token.text().starts_with("\\") {
        return handle_command_hover(token.text());
    }

    None
}


/// Handles hover for environments (equation, align, figure, table, etc.)
fn handle_environment_hover(node: &SyntaxNode) -> Option<Hover> {
    let text = node.to_string();
    
    // Extract environment name
    let env_name = if let Some(start) = text.find("\\begin{") {
        if let Some(end) = text[start..].find('}') {
            &text[start + 7..start + end]
        } else {
            "unknown"
        }
    } else {
        return None;
    };

    let (icon, description, tip) = match env_name {
        "equation" | "equation*" => (
            "∑",
            "Numbered/unnumbered display equation",
            "Press **Cmd/Ctrl+Click** on PDF to jump back to source"
        ),
        "align" | "align*" => (
            "≡",
            "Aligned multi-line equations",
            "Use `&` for alignment points, `\\\\` for line breaks"
        ),
        "gather" | "gather*" => (
            "⊕",
            "Centered multi-line equations (no alignment)",
            "Each line is independently centered"
        ),
        "figure" => (
            "🖼",
            "Floating figure environment",
            "Use `\\caption{}` and `\\label{}` for referencing"
        ),
        "table" => (
            "📊",
            "Floating table environment",
            "Use `\\caption{}` and `\\label{}` for referencing"
        ),
        "itemize" => (
            "•",
            "Bulleted list",
            "Use `\\item` for each list entry"
        ),
        "enumerate" => (
            "①",
            "Numbered list",
            "Use `\\item` for each list entry"
        ),
        "abstract" => (
            "📄",
            "Document abstract/summary",
            "Typically used after `\\maketitle`"
        ),
        _ => {
            let _desc = format!("LaTeX environment: {}", env_name);
            return Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!("📦 **`\\begin{{{}}}`**\n\nCustom environment\n\n💡 *Tip: See package documentation*", env_name),
                }),
                range: None,
            });
        }
    };

    Some(Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: format!("{} **`\\begin{{{}}}`**\n\n{}\n\n💡 *Tip: {}*", icon, env_name, description, tip),
        }),
        range: None,
    })
}

/// Handles hover for citations
fn handle_citation_hover(node: &SyntaxNode, workspace: &crate::workspace::Workspace) -> Option<Hover> {
    if let Some((keys, _)) = crate::workspace::extract_label_data(node) {
        for key in keys.split(',') {
            let key = key.trim();
            if let Some(details) = workspace.get_citation_details(key) {
                return Some(Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: details,
                    }),
                    range: None,
                });
            }
        }
        
        // Citation key not found in bibliography
        Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: format!("📚 **Citation**: `{}`\n\n⚠️ Not found in bibliography files", keys),
            }),
            range: None,
        })
    } else {
        None
    }
}

/// Handles hover for common LaTeX commands
fn handle_command_hover(text: &str) -> Option<Hover> {
    // Extract command name only (stop at { or [ or space or non-command char)
    // Commands like \section* need to keep the *
    // Commands like \section{...} need to stop at {
    
    let cmd = if let Some(idx) = text.find(['{', '[', ' ']) {
        &text[..idx]
    } else {
        text.trim()
    };
    
    // Also trim newline if somehow present (though parser usually separates)
    let cmd = cmd.trim();

    // Common document structure commands
    let (description, example) = match cmd {
        "\\section" | "\\section*" => (
            "📑 **Section heading**",
            "Numbered chapter subdivision. Use `*` for unnumbered."
        ),
        "\\subsection" | "\\subsection*" => (
            "📑 **Subsection heading**",
            "Subdivision of a section. Use `*` for unnumbered."
        ),
        "\\subsubsection" | "\\subsubsection*" => (
            "📑 **Subsubsection heading**",
            "Subdivision of a subsection. Use `*` for unnumbered."
        ),
        "\\chapter" | "\\chapter*" => (
            "📖 **Chapter heading**",
            "Top-level division (book/report classes). Use `*` for unnumbered."
        ),
        
        // Text formatting
        "\\textbf" => ("**Bold text**", "Usage: `\\textbf{text}`"),
        "\\textit" => ("*Italic text*", "Usage: `\\textit{text}`"),
        "\\texttt" => ("`Typewriter text`", "Usage: `\\texttt{code}`"),
        "\\emph" => ("*Emphasized text*", "Semantic emphasis (usually italic)"),
        "\\underline" => ("Underlined text", "Usage: `\\underline{text}`"),
        
        // Math
        "\\frac" => ("➗ Fraction", "Usage: `\\frac{numerator}{denominator}`"),
        "\\sqrt" => ("√ Square root", "Usage: `\\sqrt{x}` or `\\sqrt[n]{x}`"),
        "\\sum" => ("∑ Summation", "Usage: `\\sum_{i=1}^{n}`"),
        "\\int" => ("∫ Integral", "Usage: `\\int_{a}^{b} f(x) dx`"),
        "\\prod" => ("∏ Product", "Usage: `\\prod_{i=1}^{n}`"),
        "\\lim" => ("lim Limit", "Usage: `\\lim_{x \\to \\infty}`"),
        
        // Advanced Math (AMS)
        "\\text" => ("📝 Text in math mode", "From **amsmath**. Usage: `\\text{some text}`"),
        "\\mathbb" => ("ℝ Blackboard bold", "From **amssymb**. Usage: `\\mathbb{R}` for real numbers"),
        "\\boldsymbol" => ("𝐱 Bold math symbol", "From **amsmath**. Usage: `\\boldsymbol{x}`"),
        
        // References
        "\\label" => ("🏷 Label", "Defines a reference point for `\\ref` or `\\eqref`"),
        "\\ref" => ("🔗 Reference", "References a `\\label`"),
        "\\eqref" => ("🔗 Equation reference", "References equation with parentheses"),
        "\\cite" => ("📚 Citation", "Cites a bibliography entry"),
        "\\cref" => ("🔗 Smart reference", "From **cleveref**. Auto-adds type (Figure, Equation)"),
        
        // Graphics
        "\\includegraphics" => ("🖼 Include image", "From **graphicx**. Usage: `\\includegraphics[width=0.5\\textwidth]{file.png}`"),
        "\\graphicspath" => ("📂 Set graphics path", "From **graphicx**. Usage: `\\graphicspath{{./images/}}`"),
        
        // Colors
        "\\textcolor" => ("🎨 Colored text", "From **xcolor**. Usage: `\\textcolor{red}{text}`"),
        "\\colorbox" => ("🟦 Colored box", "From **xcolor**. Usage: `\\colorbox{blue\n}{text}`"),
        
        // Tables
        "\\toprule" => ("─ Top table rule", "From **booktabs**. Professional table lines"),
        "\\midrule" => ("─ Middle table rule", "From **booktabs**. Separates header from data"),
        "\\bottomrule" => ("─ Bottom table rule", "From **booktabs**. Clean table bottom"),
        "\\multirow" => ("🔗 Merge table rows", "From **multirow**. Usage: `\\multirow{2}{*}{text}`"),
        "\\multicolumn" => ("🔗 Merge table columns", "Usage: `\\multicolumn{2}{c}{text}`"),
        
        // Links & URLs
        "\\href" => ("🔗 Hyperlink", "From **hyperref**. Usage: `\\href{url}{text}`"),
        "\\url" => ("🌐 URL", "From **hyperref**. Usage: `\\url{https://example.com}`"),
        
        // Packages
        "\\usepackage" => ("📦 Package import", "Loads LaTeX package. Usage: `\\usepackage[options]{package}`"),
        "\\documentclass" => ("📄 Document class", "Defines document type (article, book, report, beamer)"),
        
        // Lists
        "\\item" => ("• List item", "Item in itemize/enumerate/description lists"),
        "\\setlist" => ("⚙️ Configure lists", "From **enumitem**. Customize list appearance"),
        
        // Spacing & Layout
        "\\vspace" => ("↕ Vertical space", "Usage: `\\vspace{1cm}` or `\\vspace{\\baselineskip}`"),
        "\\hspace" => ("↔ Horizontal space", "Usage: `\\hspace{1cm}` or `\\hspace{\\fill}`"),
        "\\newpage" => ("📄 Page break", "Forces a new page"),
        "\\clearpage" => ("📄 Clear page", "Flushes floats and starts new page"),
        
        // Fonts
        "\\fontsize" => ("🔤 Font size", "Usage: `\\fontsize{12pt}{14pt}\\selectfont`"),
       
        "\\textrm" => ("Roman font", "Usage: `\\textrm{text}`"),
        "\\textsf" => ("Sans-serif font", "Usage: `\\textsf{text}`"),
        
        // Quotations
        "\\enquote" => ("\" Quotation marks", "From **csquotes**. Context-sensitive quotes"),
        
        // Special
        "\\begin" => ("▶ Environment start", "Begins an environment block"),
        "\\end" => ("◀ Environment end", "Ends an environment block"),
        
        // Units
        "\\SI" => ("📏 Number with unit", "From **siunitx**. Usage: `\\SI{100}{\\meter}`"),
        "\\si" => ("📏 Unit only", "From **siunitx**. Usage: `\\si{\\kilo\\gram}`"),
        "\\num" => ("🔢 Formatted number", "From **siunitx**. Usage: `\\num{12345.67}`"),
        
        // Code
        "\\lstlisting" => ("💻 Code listing", "From **listings**. Environment for code blocks"),
        "\\verb" => ("💻 Inline verbatim", "Usage: `\\verb|code|` (delimiter can be any character)"),
        
        // Algorithms
        "\\algorithm" => ("🔄 Algorithm environment", "From **algorithm** or **algorithm2e**"),
        
        // Bibliography
        "\\bibliography" => ("📚 Bibliography file", "Specifies .bib file(s)"),
        "\\bibliographystyle" => ("📚 Bibliography style", "Sets citation style (plain, alpha, etc.)"),
        
        _ => return None, // Unknown command, no hover
    };

    Some(Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: format!("{}\n\n{}", description, example),
        }),
        range: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferrotex_syntax::parse;

    #[test]
    fn test_hover_environment() {
        let input = r#"
        \begin{equation}
            E = mc^2
        \end{equation}
        "#;
        let p = parse(input);
        let offset = TextSize::from(input.find("\\begin").unwrap() as u32);
        let workspace = crate::workspace::Workspace::default();
        let hover = find_hover(&p.syntax(), offset, &workspace).expect("No hover found");
        
        match hover.contents {
            HoverContents::Markup(m) => {
                assert_eq!(m.kind, MarkupKind::Markdown);
                assert!(m.value.contains("equation"));
                assert!(!m.value.contains("E = mc^2")); // Should NOT show raw LaTeX
            },
            _ => panic!("Wrong hover content type"),
        }
    }

    #[test]
    fn test_hover_command() {
        let input = r#"\textbf{bold text}"#;
        let p = parse(input);
        let offset = TextSize::from(input.find("textbf").unwrap() as u32);
        let workspace = crate::workspace::Workspace::default();
        let hover = find_hover(&p.syntax(), offset, &workspace);
        
        assert!(hover.is_some());
        match hover.unwrap().contents {
            HoverContents::Markup(m) => {
                assert!(m.value.contains("Bold"));
            },
            _ => panic!("Wrong hover content type"),
        }
    }

    #[test]
    fn test_hover_citation() {
        use tower_lsp::lsp_types::Url;
        let workspace = crate::workspace::Workspace::default();
        let bib_uri = Url::parse("file:///refs.bib").unwrap();
        workspace.update_bib(&bib_uri, "@article{knuth77, author={Knuth}, title={The Art}, year={1977}}");
        
        let input = r#"\cite{knuth77}"#;
        let p = parse(input);
        let offset = TextSize::from(input.find("knuth77").unwrap() as u32);
        
        let hover = find_hover(&p.syntax(), offset, &workspace).expect("No citation hover");
        match hover.contents {
            HoverContents::Markup(m) => {
                assert!(m.value.contains("Knuth"));
                assert!(m.value.contains("Art"));
            },
            _ => panic!("Wrong hover content type"),
        }
    }
}
