#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompilerMode {
    Lexer,
    ASTDump,
    Compiler,
    Assembly,
    Functions,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliOptions {
    mode: CompilerMode,
    src: String,
    output: Option<String>,
    debug_path: Option<String>,

    pub allow_syscalld: bool,
}

impl CliOptions {
    pub fn mode(&self) -> CompilerMode {
        self.mode
    }

    pub fn src(&self) -> &str {
        &self.src
    }

    pub fn debug_path(&self) -> Option<&str> {
        self.debug_path.as_deref()
    }

    pub fn output(&self) -> Option<&str> {
        self.output.as_deref()
    }

    pub fn new(args: Vec<String>) -> Self {
        let mut options = CliOptions {
            mode: CompilerMode::Compiler,
            src: String::new(),
            output: None,
            debug_path: None,

            allow_syscalld: false,
        };
        let mut iter = args.into_iter();
        loop {
            let arg = match iter.next() {
                Some(arg) => arg,
                None => break,
            };

            match arg.as_str() {
                "-l" | "--lexer" => options.mode = CompilerMode::Lexer,
                "-c" | "--compiler" => options.mode = CompilerMode::Compiler,
                "-a" | "--asm" => options.mode = CompilerMode::Assembly,
                "-A" | "--ast-dump" => options.mode = CompilerMode::ASTDump,
                "-F" | "--functions" => options.mode = CompilerMode::Functions,

                "-f" | "--file" => {
                    let filename = match iter.next() {
                        Some(filename) => filename,
                        None => {
                            println!("Expected filename following -f/--file");
                            std::process::exit(1);
                        }
                    };

                    let src = match std::fs::read_to_string(&filename) {
                        Ok(src) => src,
                        Err(err) => {
                            println!("Error reading file: {}", err);
                            std::process::exit(1);
                        }
                    };

                    options.src = src;
                }

                "-i" | "--input" => {
                    options.src = match iter.next() {
                        Some(src) => src,
                        None => {
                            println!("Expected input following -i/--input");
                            std::process::exit(1);
                        }
                    }
                }

                "-o" | "--output" => {
                    options.output = match iter.next() {
                        Some(output) => Some(output),
                        None => {
                            println!("Expected output filename following -o/--output");
                            std::process::exit(1);
                        }
                    }
                }

                "-d" | "--debug" => {
                    options.debug_path = match iter.next() {
                        Some(debug_path) => Some(debug_path),
                        None => {
                            println!("Expected debug path following -d/--debug");
                            std::process::exit(1);
                        }
                    }
                }

                "-D" | "--debug-functions" => {
                    options.debug_path = Some(String::new());
                }

                "--allow-syscalld" => options.allow_syscalld = true,

                "-h" | "--help" => {
                    println!(
                        "\
Usage: compiler [options]
Options:

-h, --help: Display this help message

To compile stdlib;
  cargo run --bin compiler -- -F -f stdlib/src/math.lav -o stdlib/math.bin --allow-syscalld
  cargo run --bin compiler -- -F -f stdlib/src/system.lav -o stdlib/system.bin --allow-syscalld

Operational Modes (mutually exclusive):
  -l, --lexer: Run the lexer
  -c, --compiler: Run the compiler
  -a, --asm: Run the assembly transcoder
  -A, --ast-dump: Run the AST dumper
  -F, --functions: Compile and dump functions

Input/Output Options:
  -f, --file <filename>: Read input from file
  -i, --input <input>: Read input from command line
  -o, --output <output>: Set output filename
  -d, --debug <path>: Set debug symbol output path
  -D, --debug-functions: Enable debug symbols, but don't output them (warning; only useful with -F)

Flags:
  --allow-syscalld: Enables calls to __syscalld() in the compiler\
"
                    );
                    std::process::exit(0);
                }

                _ => {
                    println!("Unknown option: {}", arg);
                    std::process::exit(1);
                }
            }
        }

        options
    }
}
