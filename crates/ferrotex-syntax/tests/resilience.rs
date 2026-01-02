use ferrotex_syntax::{parse, SyntaxKind};

#[test]
fn test_incomplete_environment() {
    let input = "\\begin{itemize";
    let parse = parse(input);
    let root = parse.syntax();
    
    // Should not panic.
    // Should contain a root node.
    assert_eq!(root.kind(), SyntaxKind::Root);
    
    // Should contain an incomplete Environment or Command?
    // Depending on lexer, it might be Command(\begin) + Group({itemize)
    println!("{:#?}", root);
}

#[test]
fn test_incomplete_group() {
    let input = "\\textbf{Hello";
    let parse = parse(input);
    let root = parse.syntax();
    
    assert_eq!(root.kind(), SyntaxKind::Root);
    println!("{:#?}", root);
    
    // Should cover all text
    assert_eq!(u32::from(root.text_range().len()), input.len() as u32);
}

#[test]
fn test_stray_braces() {
    let input = "\\} \\{";
    let parse = parse(input);
    let root = parse.syntax();
    assert_eq!(root.kind(), SyntaxKind::Root);
}
