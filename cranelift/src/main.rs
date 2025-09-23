use std::time::Instant;

use cranelift_example::{
    compile_add_function, compile_expression, compile_factorial, compile_fibonacci, compile_max,
    compile_quadratic, compile_sum_array, compile_with_print, Expr, JitCompiler,
};

fn main() {
    println!("Cranelift JIT Compiler Examples");
    println!("================================\n");

    let mut jit = JitCompiler::new();

    // Example 1: Simple addition function
    println!("1. Simple Addition Function:");
    let add_id = compile_add_function(&mut jit).unwrap();
    jit.finalize();

    let add_fn =
        unsafe { std::mem::transmute::<*const u8, fn(i64, i64) -> i64>(jit.get_function(add_id)) };

    println!("   add(5, 3) = {}", add_fn(5, 3));
    println!("   add(100, -50) = {}", add_fn(100, -50));

    // Example 2: Factorial function
    println!("\n2. Factorial Function:");
    let mut jit = JitCompiler::new();
    let fact_id = compile_factorial(&mut jit).unwrap();
    jit.finalize();

    let fact_fn =
        unsafe { std::mem::transmute::<*const u8, fn(i64) -> i64>(jit.get_function(fact_id)) };

    for n in 0..=10 {
        println!("   factorial({}) = {}", n, fact_fn(n));
    }

    // Example 3: Fibonacci function
    println!("\n3. Fibonacci Function:");
    let mut jit = JitCompiler::new();
    let fib_id = compile_fibonacci(&mut jit).unwrap();
    jit.finalize();

    let fib_fn =
        unsafe { std::mem::transmute::<*const u8, fn(i64) -> i64>(jit.get_function(fib_id)) };

    for n in 0..=10 {
        println!("   fibonacci({}) = {}", n, fib_fn(n));
    }

    // Example 4: Maximum function
    println!("\n4. Maximum Function:");
    let mut jit = JitCompiler::new();
    let max_id = compile_max(&mut jit).unwrap();
    jit.finalize();

    let max_fn =
        unsafe { std::mem::transmute::<*const u8, fn(i64, i64) -> i64>(jit.get_function(max_id)) };

    println!("   max(5, 3) = {}", max_fn(5, 3));
    println!("   max(2, 8) = {}", max_fn(2, 8));
    println!("   max(-10, -5) = {}", max_fn(-10, -5));

    // Example 5: Quadratic function
    println!("\n5. Quadratic Function (f(x) = ax² + bx + c):");
    let mut jit = JitCompiler::new();
    let quad_id = compile_quadratic(&mut jit).unwrap();
    jit.finalize();

    let quad_fn = unsafe {
        std::mem::transmute::<*const u8, fn(f64, f64, f64, f64) -> f64>(jit.get_function(quad_id))
    };

    println!(
        "   f(2) where a=1, b=2, c=1: {}",
        quad_fn(2.0, 1.0, 2.0, 1.0)
    );
    println!(
        "   f(3) where a=2, b=-3, c=5: {}",
        quad_fn(3.0, 2.0, -3.0, 5.0)
    );

    // Example 6: Expression compilation
    println!("\n6. Expression Compilation:");
    let mut jit = JitCompiler::new();

    // Compile: (x + 5) * (y - 3)
    let expr = Expr::Mul(
        Box::new(Expr::Add(Box::new(Expr::Var(0)), Box::new(Expr::Const(5)))),
        Box::new(Expr::Sub(Box::new(Expr::Var(1)), Box::new(Expr::Const(3)))),
    );

    println!("   Expression: (x + 5) * (y - 3)");
    let expr_id = compile_expression(&mut jit, expr).unwrap();
    jit.finalize();

    let expr_fn =
        unsafe { std::mem::transmute::<*const u8, fn(i64, i64) -> i64>(jit.get_function(expr_id)) };

    println!("   eval(x=10, y=8) = {}", expr_fn(10, 8));
    println!("   eval(x=2, y=5) = {}", expr_fn(2, 5));

    // Example 7: Array sum function
    println!("\n7. Array Sum Function:");
    let mut jit = JitCompiler::new();
    let sum_id = compile_sum_array(&mut jit).unwrap();
    jit.finalize();

    let sum_fn = unsafe {
        std::mem::transmute::<*const u8, fn(*const i64, i64) -> i64>(jit.get_function(sum_id))
    };

    let array = [1i64, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let sum = sum_fn(array.as_ptr(), array.len() as i64);
    println!("   sum([1,2,3,4,5,6,7,8,9,10]) = {}", sum);

    // Example 8: Function with print
    println!("\n8. Function with External Call (Print):");
    let mut jit = JitCompiler::new();
    let print_id = compile_with_print(&mut jit).unwrap();
    jit.finalize();

    let print_fn =
        unsafe { std::mem::transmute::<*const u8, fn(i64, i64)>(jit.get_function(print_id)) };

    print!("   print_sum(15, 27) outputs: ");
    print_fn(15, 27);

    // Example 9: Performance comparison
    println!("\n9. Performance Comparison:");

    // Native Rust factorial
    fn rust_factorial(n: i64) -> i64 {
        let mut result = 1;
        for i in 1..=n {
            result *= i;
        }
        result
    }

    // JIT compiled factorial
    let mut jit = JitCompiler::new();
    let fact_id = compile_factorial(&mut jit).unwrap();
    jit.finalize();
    let jit_factorial =
        unsafe { std::mem::transmute::<*const u8, fn(i64) -> i64>(jit.get_function(fact_id)) };

    const ITERATIONS: u32 = 1_000_000;
    const N: i64 = 20;

    // Benchmark Rust version
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        std::hint::black_box(rust_factorial(N));
    }
    let rust_time = start.elapsed();

    // Benchmark JIT version
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        std::hint::black_box(jit_factorial(N));
    }
    let jit_time = start.elapsed();

    println!(
        "   Native Rust factorial(20) {} iterations: {:?}",
        ITERATIONS, rust_time
    );
    println!(
        "   JIT compiled factorial(20) {} iterations: {:?}",
        ITERATIONS, jit_time
    );
    println!(
        "   Speedup: {:.2}x",
        rust_time.as_secs_f64() / jit_time.as_secs_f64()
    );

    // Example 10: Complex expression tree
    println!("\n10. Complex Expression Tree:");
    let mut jit = JitCompiler::new();

    // ((x * 2) + (y * 3)) - ((x + y) * 4)
    let complex_expr = Expr::Sub(
        Box::new(Expr::Add(
            Box::new(Expr::Mul(Box::new(Expr::Var(0)), Box::new(Expr::Const(2)))),
            Box::new(Expr::Mul(Box::new(Expr::Var(1)), Box::new(Expr::Const(3)))),
        )),
        Box::new(Expr::Mul(
            Box::new(Expr::Add(Box::new(Expr::Var(0)), Box::new(Expr::Var(1)))),
            Box::new(Expr::Const(4)),
        )),
    );

    println!("   Expression: ((x * 2) + (y * 3)) - ((x + y) * 4)");
    let complex_id = compile_expression(&mut jit, complex_expr).unwrap();
    jit.finalize();

    let complex_fn = unsafe {
        std::mem::transmute::<*const u8, fn(i64, i64) -> i64>(jit.get_function(complex_id))
    };

    println!("   eval(x=5, y=3) = {}", complex_fn(5, 3));
    println!("   eval(x=10, y=2) = {}", complex_fn(10, 2));

    println!("\nAll Cranelift examples completed!");
}
