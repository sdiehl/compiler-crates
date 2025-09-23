use bitflags_example::{
    demonstrate_compiler_flags, demonstrate_file_permissions, CompilerFlags, CompilerOptions,
};

fn main() {
    println!("=== File Permissions Example ===");
    demonstrate_file_permissions();

    println!();
    println!("=== Compiler Flags Example ===");
    demonstrate_compiler_flags();

    println!();
    println!("=== Compiler Options Example ===");
    let mut options = CompilerOptions::new(CompilerFlags::DEBUG);
    println!("Debug build options: {:?}", options);
    println!("Is debug build: {}", options.is_debug_build());

    options.enable_profiling();
    println!("After enabling profiling: {:?}", options);
}
