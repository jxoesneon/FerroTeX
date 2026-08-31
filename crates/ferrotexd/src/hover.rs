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
pub fn find_hover(
    root: &SyntaxNode,
    offset: TextSize,
    workspace: &crate::workspace::Workspace,
) -> Option<Hover> {
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
        return handle_command_hover(&current.to_string(), workspace);
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
                if token_text == "\\begin"
                    || token_text == "\\end"
                    || token_text == "begin"
                    || token_text == "end"
                {
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
        return handle_command_hover(token.text(), workspace);
    }

    None
}

/// Handles hover for environments (equation, align, figure, table, etc.)
fn handle_environment_hover(node: &SyntaxNode) -> Option<Hover> {
    let text = node.to_string();

    // Extract environment name
    let env_name = {
        let start = text.find("\\begin{")?;
        if let Some(end) = text[start..].find('}') {
            &text[start + 7..start + end]
        } else {
            "unknown"
        }
    };

    let (icon, description, tip) = match env_name {
        "equation" | "equation*" => (
            "∑",
            "Numbered/unnumbered display equation",
            "Press **Cmd/Ctrl+Click** on PDF to jump back to source",
        ),
        "align" | "align*" => (
            "≡",
            "Aligned multi-line equations",
            "Use `&` for alignment points, `\\\\` for line breaks",
        ),
        "gather" | "gather*" => (
            "⊕",
            "Centered multi-line equations (no alignment)",
            "Each line is independently centered",
        ),
        "figure" => (
            "🖼",
            "Floating figure environment",
            "Use `\\caption{}` and `\\label{}` for referencing",
        ),
        "table" => (
            "📊",
            "Floating table environment",
            "Use `\\caption{}` and `\\label{}` for referencing",
        ),
        "itemize" => ("•", "Bulleted list", "Use `\\item` for each list entry"),
        "enumerate" => ("①", "Numbered list", "Use `\\item` for each list entry"),
        "abstract" => (
            "📄",
            "Document abstract/summary",
            "Typically used after `\\maketitle`",
        ),
        _ => {
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
            value: format!(
                "{} **`\\begin{{{}}}`**\n\n{}\n\n💡 *Tip: {}*",
                icon, env_name, description, tip
            ),
        }),
        range: None,
    })
}

/// Handles hover for citations
fn handle_citation_hover(
    node: &SyntaxNode,
    workspace: &crate::workspace::Workspace,
) -> Option<Hover> {
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
                value: format!(
                    "📚 **Citation**: `{}`\n\n⚠️ Not found in bibliography files",
                    keys
                ),
            }),
            range: None,
        })
    } else {
        None
    }
}

/// Handles hover for common LaTeX commands and user-defined macros
fn handle_command_hover(text: &str, workspace: &crate::workspace::Workspace) -> Option<Hover> {
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

    // 1. Check user-defined macros first
    // We need to iterate over all indexed files to find the macro definition.
    // Ideally, Workspace should have a lookup method, but for now we iterate indices.
    for entry in workspace.indices.iter() {
        if let Some(macro_def) = entry.value().macros.iter().find(|m| m.name == cmd) {
            let args_sig = if macro_def.args > 0 {
                let mut s = String::new();
                if macro_def.has_optional {
                    s.push_str("[opt]");
                    for i in 2..=macro_def.args {
                        s.push_str(&format!("{{arg{}}}", i));
                    }
                } else {
                    for i in 1..=macro_def.args {
                        s.push_str(&format!("{{arg{}}}", i));
                    }
                }
                s
            } else {
                String::new()
            };

            return Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!(
                        "🔧 **User Macro**\n\n`{}{}`\n\nDefined in: `{}`",
                        cmd,
                        args_sig,
                        entry.key()
                    ),
                }),
                range: None,
            });
        }
    }

    // 2. Common document structure commands
    let (description, example) = match cmd {
        "\\section" | "\\section*" => (
            "📑 **Section heading**",
            "Numbered chapter subdivision. Use `*` for unnumbered.",
        ),
        "\\subsection" | "\\subsection*" => (
            "📑 **Subsection heading**",
            "Subdivision of a section. Use `*` for unnumbered.",
        ),
        "\\subsubsection" | "\\subsubsection*" => (
            "📑 **Subsubsection heading**",
            "Subdivision of a subsection. Use `*` for unnumbered.",
        ),
        "\\chapter" | "\\chapter*" => (
            "📖 **Chapter heading**",
            "Top-level division (book/report classes). Use `*` for unnumbered.",
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
        "\\text" => (
            "📝 Text in math mode",
            "From **amsmath**. Usage: `\\text{some text}`",
        ),
        "\\mathbb" => (
            "ℝ Blackboard bold",
            "From **amssymb**. Usage: `\\mathbb{R}` for real numbers",
        ),
        "\\boldsymbol" => (
            "𝐱 Bold math symbol",
            "From **amsmath**. Usage: `\\boldsymbol{x}`",
        ),

        // References
        "\\label" => (
            "🏷 Label",
            "Defines a reference point for `\\ref` or `\\eqref`",
        ),
        "\\ref" => ("🔗 Reference", "References a `\\label`"),
        "\\eqref" => (
            "🔗 Equation reference",
            "References equation with parentheses",
        ),
        "\\cite" => ("📚 Citation", "Cites a bibliography entry"),
        "\\cref" => (
            "🔗 Smart reference",
            "From **cleveref**. Auto-adds type (Figure, Equation)",
        ),

        // Graphics
        "\\includegraphics" => (
            "🖼 Include image",
            "From **graphicx**. Usage: `\\includegraphics[width=0.5\\textwidth]{file.png}`",
        ),
        "\\graphicspath" => (
            "📂 Set graphics path",
            "From **graphicx**. Usage: `\\graphicspath{{./images/}}`",
        ),

        // Colors
        "\\textcolor" => (
            "🎨 Colored text",
            "From **xcolor**. Usage: `\\textcolor{red}{text}`",
        ),
        "\\colorbox" => (
            "🟦 Colored box",
            "From **xcolor**. Usage: `\\colorbox{blue\n}{text}`",
        ),

        // Tables
        "\\toprule" => (
            "─ Top table rule",
            "From **booktabs**. Professional table lines",
        ),
        "\\midrule" => (
            "─ Middle table rule",
            "From **booktabs**. Separates header from data",
        ),
        "\\bottomrule" => (
            "─ Bottom table rule",
            "From **booktabs**. Clean table bottom",
        ),
        "\\multirow" => (
            "🔗 Merge table rows",
            "From **multirow**. Usage: `\\multirow{2}{*}{text}`",
        ),
        "\\multicolumn" => (
            "🔗 Merge table columns",
            "Usage: `\\multicolumn{2}{c}{text}`",
        ),

        // Links & URLs
        "\\href" => (
            "🔗 Hyperlink",
            "From **hyperref**. Usage: `\\href{url}{text}`",
        ),
        "\\url" => (
            "🌐 URL",
            "From **hyperref**. Usage: `\\url{https://example.com}`",
        ),

        // Packages
        "\\usepackage" => (
            "📦 Package import",
            "Loads LaTeX package. Usage: `\\usepackage[options]{package}`",
        ),
        "\\documentclass" => (
            "📄 Document class",
            "Defines document type (article, book, report, beamer)",
        ),

        // Lists
        "\\item" => ("• List item", "Item in itemize/enumerate/description lists"),
        "\\setlist" => (
            "⚙️ Configure lists",
            "From **enumitem**. Customize list appearance",
        ),

        // Spacing & Layout
        "\\vspace" => (
            "↕ Vertical space",
            "Usage: `\\vspace{1cm}` or `\\vspace{\\baselineskip}`",
        ),
        "\\hspace" => (
            "↔ Horizontal space",
            "Usage: `\\hspace{1cm}` or `\\hspace{\\fill}`",
        ),
        "\\newpage" => ("📄 Page break", "Forces a new page"),
        "\\clearpage" => ("📄 Clear page", "Flushes floats and starts new page"),

        // Fonts
        "\\fontsize" => (
            "🔤 Font size",
            "Usage: `\\fontsize{12pt}{14pt}\\selectfont`",
        ),

        "\\textrm" => ("Roman font", "Usage: `\\textrm{text}`"),
        "\\textsf" => ("Sans-serif font", "Usage: `\\textsf{text}`"),

        // Quotations
        "\\enquote" => (
            "\" Quotation marks",
            "From **csquotes**. Context-sensitive quotes",
        ),

        // Special
        "\\begin" => ("▶ Environment start", "Begins an environment block"),
        "\\end" => ("◀ Environment end", "Ends an environment block"),

        // Units
        "\\SI" => (
            "📏 Number with unit",
            "From **siunitx**. Usage: `\\SI{100}{\\meter}`",
        ),
        "\\si" => (
            "📏 Unit only",
            "From **siunitx**. Usage: `\\si{\\kilo\\gram}`",
        ),
        "\\num" => (
            "🔢 Formatted number",
            "From **siunitx**. Usage: `\\num{12345.67}`",
        ),

        // Code
        "\\lstlisting" => (
            "💻 Code listing",
            "From **listings**. Environment for code blocks",
        ),
        "\\verb" => (
            "💻 Inline verbatim",
            "Usage: `\\verb|code|` (delimiter can be any character)",
        ),

        // Algorithms
        "\\algorithm" => (
            "🔄 Algorithm environment",
            "From **algorithm** or **algorithm2e**",
        ),

        // Bibliography
        "\\bibliography" => ("📚 Bibliography file", "Specifies .bib file(s)"),
        "\\bibliographystyle" => (
            "📚 Bibliography style",
            "Sets citation style (plain, alpha, etc.)",
        ),

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
            }
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
            }
            _ => panic!("Wrong hover content type"),
        }
    }

    #[test]
    fn test_hover_citation() {
        use tower_lsp::lsp_types::Url;
        let workspace = crate::workspace::Workspace::default();
        let bib_uri = Url::parse("file:///refs.bib").unwrap();
        workspace.update_bib(
            &bib_uri,
            "@article{knuth77, author={Knuth}, title={The Art}, year={1977}}",
        );

        let input = r#"\cite{knuth77}"#;
        let p = parse(input);
        let offset = TextSize::from(input.find("knuth77").unwrap() as u32);

        let hover = find_hover(&p.syntax(), offset, &workspace).expect("No citation hover");
        match hover.contents {
            HoverContents::Markup(m) => {
                assert!(m.value.contains("Knuth"));
                assert!(m.value.contains("Art"));
            }
            _ => panic!("Wrong hover content type"),
        }
    }

    #[test]
    fn test_hover_environments_extensive() {
        let environments = vec![
            ("equation", "Numbered/unnumbered display equation"),
            ("equation*", "Numbered/unnumbered display equation"),
            ("align", "Aligned multi-line equations"),
            ("align*", "Aligned multi-line equations"),
            ("gather", "Centered multi-line equations"),
            ("gather*", "Centered multi-line equations"),
            ("figure", "Floating figure environment"),
            ("table", "Floating table environment"),
            ("itemize", "Bulleted list"),
            ("enumerate", "Numbered list"),
            ("abstract", "Document abstract"),
            ("mycustomenv", "Custom environment"),
        ];

        let workspace = crate::workspace::Workspace::default();

        for (env, expected_desc) in environments {
            let input = format!(r"\begin{{{}}} \end{{{}}}", env, env);
            let p = parse(&input);
            let offset = TextSize::from(input.find("\\begin").unwrap() as u32);

            let hover = find_hover(&p.syntax(), offset, &workspace)
                .unwrap_or_else(|| panic!("No hover found for environment: {}", env));

            match hover.contents {
                HoverContents::Markup(m) => {
                    assert!(
                        m.value.contains(expected_desc),
                        "Desc '{}' not found for {}",
                        expected_desc,
                        env
                    );
                    if env == "mycustomenv" {
                        assert!(m.value.contains("Custom environment"));
                    }
                }
                _ => panic!("Wrong hover content type"),
            }
        }
    }

    #[test]
    fn test_hover_commands_extensive() {
        let commands = vec![
            // Structure
            ("\\section", "Section heading"),
            ("\\section*", "Section heading"),
            ("\\subsection", "Subsection heading"),
            ("\\subsubsection", "Subsubsection heading"),
            ("\\chapter", "Chapter heading"),
            // Formatting
            ("\\textbf", "Bold text"),
            ("\\textit", "Italic text"),
            ("\\texttt", "Typewriter text"),
            ("\\emph", "Emphasized text"),
            ("\\underline", "Underlined text"),
            // Math
            ("\\frac", "Fraction"),
            ("\\sqrt", "Square root"),
            ("\\sum", "Summation"),
            ("\\int", "Integral"),
            ("\\prod", "Product"),
            ("\\lim", "Limit"),
            // AMS / Symbols
            ("\\text", "Text in math mode"),
            ("\\mathbb", "Blackboard bold"),
            ("\\boldsymbol", "Bold math symbol"),
            // References
            ("\\label", "Label"),
            ("\\ref", "Reference"),
            ("\\eqref", "Equation reference"),
            ("\\cite", "Citation"),
            ("\\cref", "Smart reference"),
            // Graphics
            ("\\includegraphics", "Include image"),
            ("\\graphicspath", "Set graphics path"),
            // Misc
            ("\\usepackage", "Package import"),
            ("\\documentclass", "Document class"),
            ("\\unknowncmd", ""), // Should return None
            // Add missing commands for coverage
            ("\\textsf", "Sans-serif font"),
            ("\\textrm", "Roman font"),
            ("\\newpage", "Page break"),
            ("\\clearpage", "Clear page"),
            ("\\hspace", "Horizontal space"),
            ("\\vspace", "Vertical space"),
            ("\\setlist", "Configure lists"),
            ("\\url", "URL"),
            ("\\href", "Hyperlink"),
            ("\\multirow", "Merge table rows"),
            ("\\multicolumn", "Merge table columns"),
            ("\\bottomrule", "Bottom table rule"),
            ("\\midrule", "Middle table rule"),
            ("\\toprule", "Top table rule"),
            ("\\colorbox", "Colored box"),
            ("\\textcolor", "Colored text"),
            ("\\graphicspath", "Set graphics path"),
            ("\\includegraphics", "Include image"),
            ("\\cref", "Smart reference"),
            ("\\eqref", "Equation reference"),
            ("\\cite", "Citation"),
            ("\\ref", "Reference"),
            ("\\label", "Label"),
            ("\\boldsymbol", "Bold math symbol"),
            ("\\mathbb", "Blackboard bold"),
            ("\\text", "Text in math mode"),
            ("\\lim", "Limit"),
            ("\\prod", "Product"),
            ("\\int", "Integral"),
            ("\\sum", "Summation"),
            ("\\sqrt", "Square root"),
            ("\\frac", "Fraction"),
            ("\\underline", "Underlined text"),
            ("\\emph", "Emphasized text"),
            ("\\texttt", "Typewriter text"),
            ("\\textit", "Italic text"),
            ("\\textbf", "Bold text"),
            ("\\chapter", "Chapter heading"),
            ("\\subsubsection", "Subsubsection heading"),
            ("\\subsection", "Subsection heading"),
            ("\\section", "Section heading"),
            ("\\enquote", "Quotation marks"),
            // ("\\begin", "Environment start"), // Handled by environment hover
            // ("\\end", "Environment end"),     // Handled by environment hover
            ("\\SI", "Number with unit"),
            ("\\si", "Unit only"),
            ("\\num", "Formatted number"),
            ("\\lstlisting", "Code listing"),
            ("\\verb", "Inline verbatim"),
            ("\\algorithm", "Algorithm environment"),
            ("\\bibliography", "Bibliography file"),
            ("\\bibliographystyle", "Bibliography style"),
        ];

        let workspace = crate::workspace::Workspace::default();

        for (cmd, expected_desc) in commands {
            // Need correct syntax for args?
            // e.g. \frac{a}{b}. But parser is robust.
            // Some commands like \sqrt might parse differently if no args?
            // Command hover usually triggers on the command token itself, args optional.

            let input = format!("{}{{content}}", cmd); // NO space to ensure correct parsing
            let p = parse(&input);

            // Find start of command (handle * properly)
            // search for the base command part
            let search_term = cmd.strip_prefix('\\').unwrap_or(cmd);
            // If cmd has *, search term includes it? find() works.

            let offset_pos = input.find(search_term).unwrap_or(0);
            // Adjust for leading \ (find returns index of search_term which excludes \)
            // input: "\section {content}". find("section") -> 1.
            // offset should be 0 (start of token).
            // Actually find_hover expects offset INSIDE the token.
            // 0 matches \. 1 matches s. Both are inside Command token.

            let offset = TextSize::from(offset_pos as u32);

            let hover = find_hover(&p.syntax(), offset, &workspace);

            if expected_desc.is_empty() {
                assert!(hover.is_none(), "Expected no hover for {}", cmd);
            } else {
                let h = hover.unwrap_or_else(|| panic!("No hover found for command {}", cmd));
                match h.contents {
                    HoverContents::Markup(m) => {
                        assert!(
                            m.value.contains(expected_desc),
                            "Desc '{}' not found for {} (got: {})",
                            expected_desc,
                            cmd,
                            m.value
                        );
                    }
                    _ => panic!("Wrong hover content type"),
                }
            }
        }
    }

    #[test]
    fn test_hover_citation_missing() {
        let workspace = crate::workspace::Workspace::default();
        let input = r#"\cite{missingref}"#;
        let p = parse(input);
        let offset = TextSize::from(input.find("missingref").unwrap() as u32);

        let hover = find_hover(&p.syntax(), offset, &workspace);

        if let Some(h) = hover {
            if let HoverContents::Markup(m) = h.contents {
                assert!(m.value.contains("missingref"));
            }
        }
    }

    #[test]
    fn test_hover_user_macro() {
        use tower_lsp::lsp_types::Url;
        let workspace = crate::workspace::Workspace::default();
        let uri = Url::parse("file:///macros.tex").unwrap();
        // Define a macro in the workspace
        workspace.update(&uri, r"\newcommand{\mycmd}[2]{Arg #1, #2}");

        // Now hover usage
        let input = r"\mycmd{a}{b}";
        let p = parse(input);
        let offset = TextSize::from(input.find("mycmd").unwrap() as u32);

        let hover = find_hover(&p.syntax(), offset, &workspace);
        assert!(hover.is_some(), "Should find user macro hover");
        match hover.unwrap().contents {
            HoverContents::Markup(m) => {
                assert!(m.value.contains("User Macro"));
                assert!(m.value.contains("\\mycmd"));
                assert!(m.value.contains("{arg1}{arg2}"));
                assert!(m.value.contains("macros.tex"));
            }
            _ => panic!("Wrong hover content type"),
        }
    }

    #[test]
    fn test_hover_between_tokens() {
        let input = r" \section{A}";
        let p = parse(input);
        let offset = TextSize::from(1); // Between space and \
        let workspace = crate::workspace::Workspace::default();

        let hover = find_hover(&p.syntax(), offset, &workspace);
        assert!(hover.is_some());
        match hover.unwrap().contents {
            HoverContents::Markup(m) => assert!(m.value.contains("Section heading")),
            _ => panic!("Wrong hover type"),
        }
    }

    #[test]
    fn test_hover_user_macro_optional() {
        use tower_lsp::lsp_types::Url;
        let workspace = crate::workspace::Workspace::default();
        let uri = Url::parse("file:///macros.tex").unwrap();
        // Define a macro with optional argument: \newcommand{\opt}[2][default]{...}
        // This corresponds to args=2, has_optional=true
        workspace.update(&uri, r"\newcommand{\myopt}[2][def]{Opt #1, #2}");

        let input = r"\myopt[val]{res}";
        let p = parse(input);
        let offset = TextSize::from(input.find("myopt").unwrap() as u32);

        let hover =
            find_hover(&p.syntax(), offset, &workspace).expect("Should find optional macro");
        match hover.contents {
            HoverContents::Markup(m) => {
                // Expected signature: \myopt[opt]{arg2}
                assert!(m.value.contains(r"\myopt[opt]{arg2}"));
                assert!(m.value.contains("macros.tex"));
            }
            _ => panic!("Wrong type"),
        }
    }
}
