use lavendeux_parser::{
    compiler::{asm_transcoder::ASMTranscoder, Compiler, CompilerOptions},
    lexer::{Lexer, Stack, Token},
    parser::{self, Node},
    traits::SerializeToBytes,
    value::StdFunctionSet,
    vm::ExecutionContext,
};

mod options;
use options::{CliOptions, CompilerMode};

fn main() {
    let options = CliOptions::new(std::env::args().skip(1).collect());
    let t = std::time::Instant::now();
    match exec_mode(options) {
        Ok(_) => {
            println!("Finished in {:?}", t.elapsed());
        }
        Err(e) => eprintln!("{}", e),
    }
}

fn exec_mode(options: CliOptions) -> Result<(), String> {
    let tokens = Lexer::new(options.src())
        .all_tokens()
        .or_else(|e| Err(e.to_string()))?;
    match options.mode() {
        CompilerMode::Lexer => {
            let tokens = tokens
                .into_iter()
                .map(|t| format!("{t:?}\n"))
                .collect::<String>();
            output_str(&options, &tokens)?;
        }
        CompilerMode::ASTDump => {
            let ast = build_ast(tokens)?;
            output_str(&options, &format!("{:#?}", ast))?;
        }
        CompilerMode::Compiler => {
            let (profile, bytecode) = compile_bytecode(&options, tokens)?.decompose();
            output_bin(&options, bytecode)?;

            if let Some(debug_path) = options.debug_path() {
                let profile = profile.serialize_into_bytes();
                write_bin(debug_path, profile)?;
            }
        }

        CompilerMode::Assembly => {
            let asm = transcode_asm(&options, tokens)?;
            output_str(&options, &asm)?;
        }

        CompilerMode::Functions => {
            let (profile, bytecode) = compile_bytecode(&options, tokens)?.decompose();
            let profile = match options.debug_path() {
                Some(_) => Some(profile),
                None => None,
            };

            // Run the bytecode to gather the functions into memory
            let mut context = ExecutionContext::new(bytecode, profile.clone());
            if let Err(e) = context.run() {
                return Err(format!("{}", e));
            }

            let mem = context.destroy();
            let functions = StdFunctionSet::from_mem(&mem);
            let bytes = functions.serialize_into_bytes();
            output_bin(&options, bytes)?;

            if let Some(profile) = profile {
                let debug_path = options.debug_path().unwrap();
                let profile = profile.serialize_into_bytes();
                write_bin(debug_path, profile)?;
            }
        }
    }

    Ok(())
}

fn output_str(options: &CliOptions, contents: &str) -> Result<(), String> {
    match options.output() {
        Some(filename) => write_out(&filename, contents),
        None => {
            println!("{}", contents);
            Ok(())
        }
    }
}

fn output_bin(options: &CliOptions, bytes: Vec<u8>) -> Result<(), String> {
    match options.output() {
        Some(filename) => write_bin(&filename, bytes),
        None => {
            let bytes = bytes
                .iter()
                .map(|b| format!("{:02X}", b))
                .collect::<Vec<String>>();
            let lines = bytes
                .chunks(16)
                .map(|l| l.join(" "))
                .collect::<Vec<String>>()
                .join("\n");

            println!("{lines}");
            Ok(())
        }
    }
}

fn write_bin(filename: &str, bytes: Vec<u8>) -> Result<(), String> {
    match std::fs::write(filename, bytes) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Error writing to file: {}", e)),
    }
}

fn write_out(filename: &str, contents: &str) -> Result<(), String> {
    write_bin(filename, contents.as_bytes().to_vec())
}

fn build_ast(tokens: Vec<Token>) -> Result<Node, String> {
    let stack = Stack::new(tokens);
    parser::build_ast(stack).or_else(|e| Err(e.to_string()))
}

fn compile_bytecode<'source>(
    options: &'source CliOptions,
    tokens: Vec<Token<'source>>,
) -> Result<Compiler<'source>, String> {
    let ast = build_ast(tokens)?;
    let mut compiler = Compiler::new(
        options.src(),
        CompilerOptions {
            allow_syscalld: options.allow_syscalld,
            debug: options.debug_path().is_some(),
        },
    );
    ast.compile(&mut compiler).or_else(|e| Err(e.to_string()))?;
    Ok(compiler)
}

fn transcode_asm(options: &CliOptions, tokens: Vec<Token>) -> Result<String, String> {
    let (profile, bytecode) = compile_bytecode(options, tokens)?.decompose();
    let transcoder = ASMTranscoder::new(&bytecode, Some(profile));
    Ok(transcoder.disassemble_as_string())
}
