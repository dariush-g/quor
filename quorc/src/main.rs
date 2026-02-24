use quorc::backend::Codegen;
use quorc::frontend::{lexer::Lexer, parser::Parser};
use quorc::midend::analyzer::TypeChecker;
use quorc::midend::mir::cfg::IRGenerator;

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

#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
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

    // 2) link (custom entry _start + explicit platform version)
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

    if !keep_asm {
        let _ = fs::remove_file(&asm);
    }

    Ok(())
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
pub fn build_link_run(
    asm_text: &str,
    workdir: &Path,
    out_name: &str,
    keep_asm: bool,
) -> io::Result<()> {
    let asm = workdir.join(format!("{out_name}.s"));
    let obj = workdir.join(format!("{out_name}.o"));
    let bin = workdir.join(out_name);

    // 1) write GAS-syntax assembly
    fs::write(&asm, asm_text)?;

    // 2) assemble with clang (GAS syntax, arm64)
    {
        let mut c = Command::new("clang");
        c.args([
            "-c",
            "-arch",
            "arm64",
            asm.to_str().unwrap(),
            "-o",
            obj.to_str().unwrap(),
        ]);
        run(&mut c, workdir)?;
    }

    // 3) link with clang (standard C runtime provides entry → _main)
    {
        let mut c = Command::new("clang");
        c.args([
            "-arch",
            "arm64",
            "-Wl,-platform_version,macos,15.0,15.0",
            "-o",
            bin.to_str().unwrap(),
            obj.to_str().unwrap(),
        ]);
        run(&mut c, workdir)?;
    }

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

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
pub fn build_link_run(
    asm_text: &str,
    workdir: impl Into<PathBuf>,
    out: &str,
    keep_asm: bool,
) -> io::Result<()> {
    let workdir = workdir.into();
    let asm = workdir.join(format!("{out}.asm"));
    let obj = workdir.join(format!("{out}.o"));

    fs::write(&asm, asm_text)?;

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

    run(
        Command::new("gcc").args(["-no-pie", obj.to_str().unwrap(), "-o".into(), out]),
        &workdir,
    )?;

    if !keep_asm {
        let _ = fs::remove_file(&asm);
    }

    Ok(())
}

#[cfg(all(target_os = "linux", target_arch = "aarch64"))]
pub fn build_link_run(
    asm_text: &str,
    workdir: impl Into<PathBuf>,
    out: &str,
    keep_asm: bool,
) -> io::Result<()> {
    let workdir = workdir.into();
    let asm = workdir.join(format!("{out}.s"));
    let obj = workdir.join(format!("{out}.o"));

    fs::write(&asm, asm_text)?;

    // Assemble GAS syntax
    run(
        Command::new("as").args([asm.to_str().unwrap(), "-o", obj.to_str().unwrap()]),
        &workdir,
    )?;

    // Link with gcc
    run(
        Command::new("gcc").args([obj.to_str().unwrap(), "-o".into(), out]),
        &workdir,
    )?;

    if !keep_asm {
        let _ = fs::remove_file(&asm);
    }

    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: quor <source-file>");
        std::process::exit(1);
    }

    let compiler_args = &args[2..args.len()];

    let mut src_path = PathBuf::from(&args[1]);
    if src_path.is_relative() {
        src_path = env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(&src_path);
    }

    let src_path = match fs::canonicalize(&src_path) {
        Ok(p) => p,
        Err(_) => src_path,
    };

    let workdir = src_path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    let out_name = src_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("out");

    let source = match fs::read_to_string(&src_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to read {}: {e}", src_path.display());
            std::process::exit(1);
        }
    };

    let keep_asm = source.contains("@keep_asm");

    let mut lexer = Lexer::new(source);
    let tokens = match lexer.tokenize() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Lexer error: {e:?}");
            std::process::exit(1);
        }
    };

    if compiler_args.contains(&"--emit-tokens".to_string()) {
        println!("{:?}", tokens);
    }

    let mut parser = Parser::new(tokens);
    let program = match parser.parse() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Parser error: {e:?}");
            std::process::exit(1);
        }
    };

    if compiler_args.contains(&"--emit-ast".to_string()) {
        println!("{:?}", program);
    }

    // println!("{program:?}");

    let typed = match TypeChecker::analyze_program(program, &src_path) {
        Ok(tp) => tp,
        Err(e) => {
            eprintln!("Type error: {e:?}");
            std::process::exit(1);
        }
    };

    // println!("{typed:?}");

    if compiler_args.contains(&"--emit-typed".to_string()) {
        println!("{:?}", typed);
    }

    let cfged = match IRGenerator::generate(typed) {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Type error: {e:?}");
            std::process::exit(1);
        }
    };

    if compiler_args.contains(&"--emit-mir".to_string()) {
        println!("{:?}", cfged);
    }

    let codegen = Codegen::generate(cfged);
    let asm = codegen;

    if compiler_args.contains(&"--emit-asm".to_string()) {
        println!("{}", asm);
    }

    // for st in codegen.1 {
    //     gen_asm(st, asm.clone());
    // }

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
//     let source = match fs::read_to_string(&src_path) {
//         Ok(s) => s,
//         Err(e) => {
//             eprintln!("Failed to read {}: {e}", src_path.display());
//             std::process::exit(1);
//         }
//     };

//     let mut lexer = Lexer::new(source);
//     let tokens = match lexer.tokenize() {
//         Ok(t) => t,
//         Err(e) => {
//             eprintln!("Lexer error: {e:?}");
//             std::process::exit(1);
//         }
//     };

//     let mut parser = Parser::new(tokens);
//     let program = match parser.parse() {
//         Ok(p) => p,
//         Err(e) => {
//             eprintln!("Parser error: {e:?}");
//             std::process::exit(1);
//         }
//     };

//     // println!("{program:?}");

//     let typed = match TypeChecker::analyze_program(program) {
//         Ok(tp) => tp,
//         Err(e) => {
//             eprintln!("Type error: {e:?}");
//             std::process::exit(1);
//         }
//     };

//     let codegen = CodeGen::generate(&typed);

//     current.push_str(&codegen);

//     for path in codegen {
//         gen_asm(path, current.clone());
//     }

//     current
// }
