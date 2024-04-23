#[allow(unused_macros)]
macro_rules! generate_benches {
    ($input_file:literal) => {
        use lavendeux_parser::lexer::Stack;
        use lavendeux_parser::lexer::{Lexer, Token};
        use lavendeux_parser::parser;
        use lavendeux_parser::parser::ParserNode;
        const INPUT: &'static str = include_str!($input_file);

        fn lexer_pass() -> Vec<Token<'static>> {
            Lexer::new(INPUT).all_tokens().unwrap()
        }

        fn compiler_pass1(mut stack: Stack) {
            parser::core::ScriptNode::parse(&mut stack).unwrap();
        }

        fn criterion_benchmark(c: &mut Criterion) {
            c.bench_function("Pass 1: Lexer", |b| b.iter(|| lexer_pass()));

            c.bench_function("Pass 2: Compiler phase-1", |b| {
                b.iter(|| {
                    let stack = Stack::new(lexer_pass());
                    compiler_pass1(black_box(stack))
                })
            });
        }
    };
}
