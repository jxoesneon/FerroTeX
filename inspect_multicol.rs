use ferrotex_syntax::parse;

fn main() {
    let input = r"\begin{pmatrix} \multicolumn{2}{c}{1} & 3 \\ 4 & 5 \end{pmatrix}";
    let parse = parse(input);
    println!("{:#?}", parse.syntax());
}
