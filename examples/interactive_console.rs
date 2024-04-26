use lavendeux_parser::Lavendeux;

const MIN_STACK_SIZE: usize = 32 * 1024 * 1024;

fn main() {
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

    let mut lavendeux = Lavendeux::new();
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
            let input = cmd.as_str();
            match lavendeux.run(input) {
                Ok(value) => {
                    println!("{}\n", value);
                }
                Err(e) => {
                    eprintln!("{}\n", e);
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
