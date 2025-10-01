//! Dynamic assembler for runtime ARM64 code generation.
//!
//! Demonstrates using dynasm-rs for JIT compilation and dynamic code generation
//! on ARM64 (AArch64) architecture.

use std::{io, slice};

use dynasmrt::{dynasm, DynasmApi, DynasmLabelApi, ExecutableBuffer};

/// Generates a simple "Hello World" function using ARM64 assembly.
///
/// This example demonstrates:
/// - Embedding data directly in the assembly
/// - Using labels for addressing
/// - Calling external Rust functions from assembly
/// - Stack management for ARM64 ABI
pub fn generate_hello_world() -> ExecutableBuffer {
    let mut ops = dynasmrt::aarch64::Assembler::new().unwrap();
    let string = "Hello World!";

    // Embed the string data with a label
    dynasm!(ops
        ; .arch aarch64
        ; ->hello:
        ; .bytes string.as_bytes()
    );

    // Generate the function that prints the string
    dynasm!(ops
        ; .arch aarch64
        ; adr x0, ->hello                 // Load string address into first arg (ARM64 ABI)
        ; mov w1, string.len() as u32     // Load string length into second arg
        ; movz x2, (print as usize as u64) as u32
        ; movk x2, (print as usize as u64 >> 16) as u32, lsl 16
        ; movk x2, (print as usize as u64 >> 32) as u32, lsl 32
        ; movk x2, (print as usize as u64 >> 48) as u32, lsl 48
        ; blr x2                          // Call the print function
        ; ret                             // Return
    );

    ops.finalize().unwrap()
}

/// External function called from assembly to print a buffer.
///
/// # Safety
///
/// The caller must ensure that:
/// - `buffer` points to valid memory for at least `length` bytes
/// - The memory pointed to by `buffer` is initialized
pub unsafe extern "C" fn print(buffer: *const u8, length: u64) -> bool {
    io::Write::write_all(
        &mut io::stdout(),
        slice::from_raw_parts(buffer, length as usize),
    )
    .is_ok()
}

/// Generates an optimized addition function for two integers.
///
/// Creates machine code equivalent to: `fn add(a: i32, b: i32) -> i32 { a + b
/// }`
pub fn generate_add_function() -> ExecutableBuffer {
    let mut ops = dynasmrt::aarch64::Assembler::new().unwrap();

    dynasm!(ops
        ; .arch aarch64
        ; add w0, w0, w1  // Add w1 to w0, result in w0 (ARM64 ABI)
        ; ret             // Return with result in w0
    );

    ops.finalize().unwrap()
}

/// Generates a factorial function using recursion.
///
/// Demonstrates more complex control flow with conditional jumps and recursive
/// calls.
pub fn generate_factorial() -> ExecutableBuffer {
    let mut ops = dynasmrt::aarch64::Assembler::new().unwrap();

    let entry_label = ops.new_dynamic_label();
    ops.dynamic_label(entry_label);

    dynasm!(ops
        ; .arch aarch64
        ; cmp w0, #1                      // Compare n with 1
        ; b.le ->base_case                // Jump if n <= 1
        ; stp x29, x30, [sp, #-16]!       // Save frame pointer and link register
        ; stp x19, x20, [sp, #-16]!       // Save callee-saved registers
        ; mov w19, w0                     // Save n in w19
        ; sub w0, w0, #1                  // n - 1
        ; adr x1, =>entry_label           // Load our own address for recursion
        ; blr x1                          // Recursive call with n-1
        ; mul w0, w0, w19                 // Multiply result by n
        ; ldp x19, x20, [sp], #16         // Restore callee-saved registers
        ; ldp x29, x30, [sp], #16         // Restore frame pointer and link register
        ; ret
        ; ->base_case:
        ; mov w0, #1                      // Return 1 for base case
        ; ret
    );

    ops.finalize().unwrap()
}

/// Generates a loop that sums an array of integers.
///
/// Takes a pointer to an i32 array and its length, returns the sum.
pub fn generate_array_sum() -> ExecutableBuffer {
    let mut ops = dynasmrt::aarch64::Assembler::new().unwrap();

    dynasm!(ops
        ; .arch aarch64
        ; mov w2, #0                      // Initialize sum to 0
        ; cbz x1, ->done                  // If length is 0, return 0
        ; ->loop_start:
        ; ldr w3, [x0], #4                // Load element and increment pointer
        ; add w2, w2, w3                  // Add to sum
        ; sub x1, x1, #1                  // Decrement counter
        ; cbnz x1, ->loop_start           // Continue if not zero
        ; ->done:
        ; mov w0, w2                      // Move result to return register
        ; ret
    );

    ops.finalize().unwrap()
}

/// Generates a function that performs SIMD operations using NEON instructions.
///
/// Adds two integer vectors element by element.
pub fn generate_vector_add() -> ExecutableBuffer {
    let mut ops = dynasmrt::aarch64::Assembler::new().unwrap();

    dynasm!(ops
        ; .arch aarch64
        ; ldp x3, x4, [x0]                // Load first two elements from vector 1
        ; ldp x5, x6, [x1]                // Load first two elements from vector 2
        ; add x3, x3, x5                  // Add first elements
        ; add x4, x4, x6                  // Add second elements
        ; stp x3, x4, [x2]                // Store result
        ; ret
    );

    ops.finalize().unwrap()
}

/// Demonstrates conditional compilation based on runtime values.
///
/// Generates specialized code for specific constant values.
pub fn generate_multiply_by_constant(constant: i32) -> ExecutableBuffer {
    let mut ops = dynasmrt::aarch64::Assembler::new().unwrap();

    // Optimize for powers of two using shifts
    if constant > 0 && (constant & (constant - 1)) == 0 {
        let shift = constant.trailing_zeros();
        dynasm!(ops
            ; .arch aarch64
            ; lsl w0, w0, shift              // Shift left for power of 2
            ; ret
        );
    } else {
        dynasm!(ops
            ; .arch aarch64
            ; mov w1, constant as u32        // Load constant
            ; mul w0, w0, w1                 // Multiply
            ; ret
        );
    }

    ops.finalize().unwrap()
}

/// Generates a memcpy implementation optimized for small sizes.
///
/// Uses a simple byte copy loop for ARM64.
pub fn generate_memcpy() -> ExecutableBuffer {
    let mut ops = dynasmrt::aarch64::Assembler::new().unwrap();

    dynasm!(ops
        ; .arch aarch64
        ; mov x3, x0                      // Save destination for return
        ; cbz x2, ->done                  // If count is 0, return
        ; ->copy_loop:
        ; ldrb w4, [x1], #1               // Load byte from source and increment
        ; strb w4, [x0], #1               // Store byte to dest and increment
        ; sub x2, x2, #1                  // Decrement count
        ; cbnz x2, ->copy_loop            // Continue if not zero
        ; ->done:
        ; mov x0, x3                      // Return original destination
        ; ret
    );

    ops.finalize().unwrap()
}

/// Helper function to execute generated code safely.
///
/// Converts the generated bytes into an executable function pointer.
///
/// # Safety
///
/// The caller must ensure that:
/// - `code` contains valid machine code for the target architecture
/// - The code follows the expected calling convention
/// - The function pointer type matches the actual generated code signature
pub unsafe fn execute_generated_code<F, R>(code: &[u8], f: F) -> R
where
    F: FnOnce(*const u8) -> R, {
    f(code.as_ptr())
}

#[cfg(test)]
#[allow(unused_unsafe)]
mod tests {
    use std::mem;

    use super::*;

    #[test]
    fn test_add_function() {
        let code = generate_add_function();
        let add_fn: extern "C" fn(i32, i32) -> i32 = unsafe { mem::transmute(code.as_ptr()) };

        assert_eq!(unsafe { add_fn(5, 3) }, 8);
        assert_eq!(unsafe { add_fn(-10, 20) }, 10);
    }

    #[test]
    fn test_factorial() {
        let code = generate_factorial();
        let factorial_fn: extern "C" fn(i32) -> i32 = unsafe { mem::transmute(code.as_ptr()) };

        assert_eq!(unsafe { factorial_fn(0) }, 1);
        assert_eq!(unsafe { factorial_fn(1) }, 1);
        assert_eq!(unsafe { factorial_fn(5) }, 120);
    }

    #[test]
    fn test_array_sum() {
        let code = generate_array_sum();
        let sum_fn: extern "C" fn(*const i32, usize) -> i32 =
            unsafe { mem::transmute(code.as_ptr()) };

        let array = [1, 2, 3, 4, 5];
        assert_eq!(unsafe { sum_fn(array.as_ptr(), array.len()) }, 15);

        let empty: [i32; 0] = [];
        assert_eq!(unsafe { sum_fn(empty.as_ptr(), 0) }, 0);
    }

    #[test]
    fn test_multiply_by_constant() {
        // Test power of two (uses shift)
        let code = generate_multiply_by_constant(8);
        let mul_fn: extern "C" fn(i32) -> i32 = unsafe { mem::transmute(code.as_ptr()) };
        assert_eq!(unsafe { mul_fn(5) }, 40);

        // Test non-power of two (uses imul)
        let code = generate_multiply_by_constant(7);
        let mul_fn: extern "C" fn(i32) -> i32 = unsafe { mem::transmute(code.as_ptr()) };
        assert_eq!(unsafe { mul_fn(6) }, 42);
    }
}
