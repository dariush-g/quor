use quor::analyzer::TypeChecker;
use quor::codegen::CodeGen;
use quor::lexer::Lexer;
use quor::parser::Parser;

use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

// nasm -f elf64 test.asm -o test.o && ld test.o -o test && ./test

// nasm -f macho64 test.asm -o test.o && clang -arch x86_64 -nostartfiles -Wl,-e,_start -o test test.o && ./test

// nasm -f elf64 test.asm -o test.o && clang -arch x86_64 -c runtime.c -o runtime.o && clang -arch x86_64 -nostartfiles -Wl,-e,_start -o test test.o runtime.o && ./test

#[cfg(target_os = "macos")]
fn run(cmd: &mut Command, workdir: &Path) -> io::Result<()> {
    cmd.env("MACOSX_DEPLOYMENT_TARGET", "15.0")
        .current_dir(workdir)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    let status = cmd.status()?;

    if !status.success() {
        return Err(io::Error::other(format!(
            "command failed with status: {status:?}"
        )));
    }

    Ok(())
}

#[cfg(target_os = "macos")]
pub fn build_link_run(
    asm_text: &str,
    workdir: &Path, // where intermediates should go
    out_name: &str, // output binary name (no extension)
    keep_asm: bool,
) -> io::Result<()> {
    let asm = workdir.join(format!("{out_name}.asm"));
    let obj = workdir.join(format!("{out_name}.o"));

    let bin = workdir.join(out_name);

    // 1) write asm
    fs::write(&asm, asm_text)?;

    {
        let mut c = Command::new("nasm");
        c.args([
            "-f",
            "macho64",
            asm.to_str().unwrap(),
            "-o",
            obj.to_str().unwrap(),
        ]);
        run(&mut c, workdir)?;
    }

    // 3) clang → runtime.o (compile for x86_64 with a consistent min version)
    // {
    //     let mut c = Command::new("clang");
    //     c.args([
    //         "-arch",
    //         "x86_64",
    //         "-mmacosx-version-min=15.0",

    // "-c",
    // rt_c.to_str().unwrap(),
    // "-o",
    // rt_o.to_str().unwrap(),
    //     ]);
    //     run(&mut c, workdir)?;
    // }

    // 4) link (custom entry _start + explicit platform version)
    {
        let mut c = Command::new("clang");
        c.args([
            "-arch",
            "x86_64",
            "-nostartfiles",
            "-Wl,-e,_start",
            "-Wl,-platform_version,macos,15.0,15.0",
            "-o",
            bin.to_str().unwrap(),
            obj.to_str().unwrap(),
        ]);
        run(&mut c, workdir)?;
    }

    // 5) run the produced binary
    // {
    //     let mut c = Command::new(&bin);
    //     run(&mut c, workdir)?;
    // }

    if !keep_asm {
        let _ = fs::remove_file(&asm);
    }

    Ok(())
}

#[cfg(target_os = "linux")]
fn run(cmd: &mut Command, workdir: &Path) -> io::Result<()> {
    // eprintln!("$ (cd {}); {:?}", workdir.display(), cmd);
    let status = cmd
        .current_dir(workdir)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;
    if !status.success() {
        return Err(io::Error::other(format!("command failed: {status:?}")));
    }
    Ok(())
}

#[cfg(target_os = "linux")]
pub fn build_link_run(
    asm_text: &str,
    workdir: impl Into<PathBuf>,
    out: &str,
    keep_asm: bool,
) -> io::Result<()> {
    let workdir = workdir.into();
    let asm = workdir.join(format!("{out}.asm"));
    let obj = workdir.join(format!("{out}.o"));

    // let bin = workdir.join(out);
    // 1) Write asm file
    fs::write(&asm, asm_text)?;

    // 2) NASM -> object
    run(
        Command::new("nasm").args([
            "-f",
            "elf64",
            asm.to_str().unwrap(),
            "-o",
            obj.to_str().unwrap(),
        ]),
        &workdir,
    )?;
    //  -nostartfiles -no-pie
    // GCC
    run(
        Command::new("gcc").args([
            // "./stdlib/read_file.o",
            //"-nostartfiles",
            "-no-pie",
            obj.to_str().unwrap(),
            "-o".into(),
            out,
        ]),
        &workdir,
    )?;

    if !keep_asm {
        let _ = fs::remove_file(&asm);
    }

    Ok(())
}

fn main() {
    // CLI: quor <source-file>
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: quor <source-file>");
        std::process::exit(1);
    }

    // Resolve source path robustly to support invocation from any directory
    // 1) Build a path from the argument (relative or absolute)
    let mut src_path = PathBuf::from(&args[1]);
    if src_path.is_relative() {
        src_path = env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(&src_path);
    }
    // 2) Canonicalize when possible (if the file exists)
    let src_path = match fs::canonicalize(&src_path) {
        Ok(p) => p,
        Err(_) => src_path, // fall back to the non-canonical path; error will surface on read
    };

    // Workdir = directory of the source file; output name = file stem
    let workdir = src_path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    let out_name = src_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("out");

    // Read source
    let source = match fs::read_to_string(&src_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to read {}: {e}", src_path.display());
            std::process::exit(1);
        }
    };

    let keep_asm = source.contains("@keep_asm");

    // Lex
    let mut lexer = Lexer::new(source);
    let tokens = match lexer.tokenize() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Lexer error: {e:?}");
            std::process::exit(1);
        }
    };

    // println!("{tokens:?}");

    // Parse
    let mut parser = Parser::new(tokens);
    let program = match parser.parse() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Parser error: {e:?}");
            std::process::exit(1);
        }
    };

    println!("{program:?}");

    // Type check
    let typed = match TypeChecker::analyze_program(program, &src_path) {
        Ok(tp) => tp,
        Err(e) => {
            eprintln!("Type error: {e:?}");
            std::process::exit(1);
        }
    };

    // println!("{typed:?}");

    // Codegen -> ASM
    let codegen = CodeGen::generate(&typed);

    let asm = codegen;

    // for st in codegen.1 {
    //     gen_asm(st, asm.clone());
    // }

    // Build → link → run
    // Adjust runtime path if yours lives elsewhere:

    if let Err(e) = build_link_run(&asm, &workdir, out_name, keep_asm) {
        eprintln!("build failed: {e}");
        std::process::exit(1);
    }
}

// fn gen_asm(input_path: String, mut current: String) -> String {
//     let mut src_path = PathBuf::from(input_path.clone());

//     if !input_path.contains("/") {
//         src_path = PathBuf::from(format!("./{}", &input_path));
//     }

//     // Read source
//     let source = match fs::read_to_string(&src_path) {
//         Ok(s) => s,
//         Err(e) => {
//             eprintln!("Failed to read {}: {e}", src_path.display());
//             std::process::exit(1);
//         }
//     };

//     // Lex
//     let mut lexer = Lexer::new(source);
//     let tokens = match lexer.tokenize() {
//         Ok(t) => t,
//         Err(e) => {
//             eprintln!("Lexer error: {e:?}");
//             std::process::exit(1);
//         }
//     };

//     // Parse
//     let mut parser = Parser::new(tokens);
//     let program = match parser.parse() {
//         Ok(p) => p,
//         Err(e) => {
//             eprintln!("Parser error: {e:?}");
//             std::process::exit(1);
//         }
//     };

//     // println!("{program:?}");

//     // Type check
//     let typed = match TypeChecker::analyze_program(program) {
//         Ok(tp) => tp,
//         Err(e) => {
//             eprintln!("Type error: {e:?}");
//             std::process::exit(1);
//         }
//     };

//     // Codegen → ASM
//     let codegen = CodeGen::generate(&typed);

//     current.push_str(&codegen);

//     for path in codegen {
//         gen_asm(path, current.clone());
//     }

//     current
// }
