use lavendeux_parser::{
    lexer::Lexer,
    lexer::Stack,
    parser::{self, ParserNode},
};

const MIN_STACK_SIZE: usize = 32 * 1024 * 1024;

fn main() {
    let input = std::fs::read_to_string("example_scripts/zarbans_grotto.lav").unwrap();
    let tokens = Lexer::new(&input).all_tokens();
    let mut tokens = Stack::new(tokens);
    let _ast =
        parser::core::ScriptNode::parse(&mut tokens).expect("Could not parse zarbans_grotto.lav");

    stacker::grow(MIN_STACK_SIZE, || interactive_compiler())
}

fn interactive_compiler() {
    // Preload command stack from arguments
    let mut stack: Vec<String> = std::env::args().skip(1).collect();
    if stack.is_empty() {
        println!("Ready! Type expressions below!");
    } else {
        stack.insert(0, "exit".to_string());
    }

    loop {
        // Make sure we have a command ready
        if stack.is_empty() {
            stack.push(next_command());
        }
        let cmd = stack.pop().unwrap();

        if cmd.is_empty() {
            continue;
        } else if ["exit", "quit"].contains(&cmd.as_str()) {
            break;
        } else {
            // Process the next command
            let t = std::time::Instant::now();
            let input = cmd.as_str();

            let tokens = Lexer::new(input).all_tokens();
            println!("{tokens:#?}");
            let mut tokens = Stack::new(tokens);
            match parser::core::ScriptNode::parse(&mut tokens) {
                Some(ast) => {
                    println!("Time: {:?}", t.elapsed());
                    println!("{:#?}", ast);
                }
                None => {
                    println!("Error: {}", tokens.emit_err());
                }
            }
        }
    }
}

fn next_command() -> String {
    let mut input = String::new();
    print!("> ");
    let _ = std::io::Write::flush(&mut std::io::stdout());

    loop {
        std::io::stdin()
            .read_line(&mut input)
            .expect("error: unable to read user input");
        if !input.trim().ends_with('\\') || input.trim().ends_with("\\\\") {
            break;
        }
    }

    return input.trim().to_string();
}
