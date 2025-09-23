use std::collections::HashMap;

use cranelift::codegen::ir::types::*;
use cranelift::codegen::ir::{AbiParam, Function, InstBuilder, Signature, UserFuncName};
use cranelift::codegen::settings::{self, Configurable};
use cranelift::codegen::verifier::verify_function;
use cranelift::codegen::Context;
use cranelift::frontend::{FunctionBuilder, FunctionBuilderContext, Variable};
use cranelift::prelude::*;
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{FuncId, Linkage, Module};

/// A simple JIT compiler using Cranelift
pub struct JitCompiler {
    builder_context: FunctionBuilderContext,
    ctx: Context,
    module: JITModule,
}

impl JitCompiler {
    pub fn new() -> Self {
        let mut flag_builder = settings::builder();
        flag_builder.set("use_colocated_libcalls", "false").unwrap();
        flag_builder.set("is_pic", "false").unwrap();
        let isa_builder = cranelift_native::builder().unwrap_or_else(|msg| {
            panic!("host machine is not supported: {}", msg);
        });
        let isa = isa_builder
            .finish(settings::Flags::new(flag_builder))
            .unwrap();

        let mut builder = JITBuilder::with_isa(isa, cranelift_module::default_libcall_names());
        builder.symbol_lookup_fn(Box::new(|name| {
            // Hook up external functions
            match name {
                "println_i64" => Some(println_i64 as *const u8),
                "println_f64" => Some(println_f64 as *const u8),
                _ => None,
            }
        }));

        let module = JITModule::new(builder);

        Self {
            builder_context: FunctionBuilderContext::new(),
            ctx: module.make_context(),
            module,
        }
    }

    pub fn compile_function(
        &mut self,
        name: &str,
        params: Vec<Type>,
        returns: Vec<Type>,
        build_fn: impl FnOnce(&mut FunctionBuilder, &[Variable]),
    ) -> Result<FuncId, String> {
        // Clear the context
        self.ctx.func = Function::with_name_signature(
            UserFuncName::user(0, 0),
            self.make_signature(params.clone(), returns.clone()),
        );

        // Create the function builder
        let mut builder = FunctionBuilder::new(&mut self.ctx.func, &mut self.builder_context);

        // Create variables for parameters
        let variables: Vec<Variable> = params.iter().map(|ty| builder.declare_var(*ty)).collect();

        // Create entry block and append parameters
        let entry_block = builder.create_block();
        builder.append_block_params_for_function_params(entry_block);
        builder.switch_to_block(entry_block);
        builder.seal_block(entry_block);

        // Define parameters
        for (i, var) in variables.iter().enumerate() {
            let val = builder.block_params(entry_block)[i];
            builder.def_var(*var, val);
        }

        // Call the user's function to build the body
        build_fn(&mut builder, &variables);

        // Finalize the function
        builder.finalize();

        // Verify the function
        if let Err(errors) = verify_function(&self.ctx.func, self.module.isa()) {
            return Err(format!("Function verification failed: {}", errors));
        }

        // Define the function in the module
        let func_id = self
            .module
            .declare_function(name, Linkage::Export, &self.ctx.func.signature)
            .map_err(|e| e.to_string())?;

        self.module
            .define_function(func_id, &mut self.ctx)
            .map_err(|e| e.to_string())?;

        // Clear the context for next use
        self.module.clear_context(&mut self.ctx);

        Ok(func_id)
    }

    pub fn finalize(&mut self) {
        self.module.finalize_definitions().unwrap();
    }

    pub fn get_function(&self, func_id: FuncId) -> *const u8 {
        self.module.get_finalized_function(func_id)
    }

    fn make_signature(&self, params: Vec<Type>, returns: Vec<Type>) -> Signature {
        let mut sig = self.module.make_signature();
        for param in params {
            sig.params.push(AbiParam::new(param));
        }
        for ret in returns {
            sig.returns.push(AbiParam::new(ret));
        }
        sig
    }
}

impl Default for JitCompiler {
    fn default() -> Self {
        Self::new()
    }
}

// External functions that can be called from JIT code
extern "C" fn println_i64(x: i64) {
    println!("{}", x);
}

extern "C" fn println_f64(x: f64) {
    println!("{}", x);
}

/// Example: Compile a simple arithmetic function
pub fn compile_add_function(jit: &mut JitCompiler) -> Result<FuncId, String> {
    jit.compile_function("add", vec![I64, I64], vec![I64], |builder, params| {
        let x = builder.use_var(params[0]);
        let y = builder.use_var(params[1]);
        let sum = builder.ins().iadd(x, y);
        builder.ins().return_(&[sum]);
    })
}

/// Example: Compile a factorial function
pub fn compile_factorial(jit: &mut JitCompiler) -> Result<FuncId, String> {
    jit.compile_function("factorial", vec![I64], vec![I64], |builder, params| {
        let n = params[0];

        // Create blocks
        let header_block = builder.create_block();
        let body_block = builder.create_block();
        let exit_block = builder.create_block();

        // Add block parameters
        builder.append_block_param(header_block, I64); // i
        builder.append_block_param(header_block, I64); // result

        // Entry: jump to header with initial values
        let one = builder.ins().iconst(I64, 1);
        builder.ins().jump(header_block, &[one.into(), one.into()]);

        // Header block: check if i <= n
        builder.switch_to_block(header_block);
        let i = builder.block_params(header_block)[0];
        let result = builder.block_params(header_block)[1];
        let n_val = builder.use_var(n);
        let cmp = builder.ins().icmp(IntCC::SignedLessThanOrEqual, i, n_val);
        builder.ins().brif(cmp, body_block, &[], exit_block, &[]);

        // Body block: result *= i; i++
        builder.switch_to_block(body_block);
        builder.seal_block(body_block);
        let new_result = builder.ins().imul(result, i);
        let new_i = builder.ins().iadd_imm(i, 1);
        builder
            .ins()
            .jump(header_block, &[new_i.into(), new_result.into()]);

        // Exit block: return result
        builder.switch_to_block(exit_block);
        builder.seal_block(exit_block);
        builder.seal_block(header_block);
        builder.ins().return_(&[result]);
    })
}

/// Example: Compile a Fibonacci function
pub fn compile_fibonacci(jit: &mut JitCompiler) -> Result<FuncId, String> {
    jit.compile_function("fibonacci", vec![I64], vec![I64], |builder, params| {
        let n = params[0];

        // Create blocks
        let check_base = builder.create_block();
        let recursive = builder.create_block();
        let return_n = builder.create_block();

        // Jump to check_base
        builder.ins().jump(check_base, &[]);

        // Check if n <= 1
        builder.switch_to_block(check_base);
        let n_val = builder.use_var(n);
        let one = builder.ins().iconst(I64, 1);
        let cmp = builder.ins().icmp(IntCC::SignedLessThanOrEqual, n_val, one);
        builder.ins().brif(cmp, return_n, &[], recursive, &[]);

        // Return n for base case
        builder.switch_to_block(return_n);
        builder.seal_block(return_n);
        builder.ins().return_(&[n_val]);

        // Recursive case: fib(n-1) + fib(n-2)
        builder.switch_to_block(recursive);
        builder.seal_block(recursive);
        builder.seal_block(check_base);

        // For simplicity, we'll compute iteratively
        let two = builder.ins().iconst(I64, 2);
        let a = builder.ins().iconst(I64, 0);
        let b = builder.ins().iconst(I64, 1);

        // Create loop blocks
        let loop_header = builder.create_block();
        let loop_body = builder.create_block();
        let loop_exit = builder.create_block();

        builder.append_block_param(loop_header, I64); // counter
        builder.append_block_param(loop_header, I64); // a
        builder.append_block_param(loop_header, I64); // b

        builder
            .ins()
            .jump(loop_header, &[two.into(), a.into(), b.into()]);

        // Loop header: check if counter <= n
        builder.switch_to_block(loop_header);
        let counter = builder.block_params(loop_header)[0];
        let curr_a = builder.block_params(loop_header)[1];
        let curr_b = builder.block_params(loop_header)[2];
        let cmp = builder
            .ins()
            .icmp(IntCC::SignedLessThanOrEqual, counter, n_val);
        builder.ins().brif(cmp, loop_body, &[], loop_exit, &[]);

        // Loop body: compute next fibonacci number
        builder.switch_to_block(loop_body);
        builder.seal_block(loop_body);
        let next_fib = builder.ins().iadd(curr_a, curr_b);
        let next_counter = builder.ins().iadd_imm(counter, 1);
        builder.ins().jump(
            loop_header,
            &[next_counter.into(), curr_b.into(), next_fib.into()],
        );

        // Loop exit: return b
        builder.switch_to_block(loop_exit);
        builder.seal_block(loop_exit);
        builder.seal_block(loop_header);
        builder.ins().return_(&[curr_b]);
    })
}

/// Example: Working with floating point
pub fn compile_quadratic(jit: &mut JitCompiler) -> Result<FuncId, String> {
    jit.compile_function(
        "quadratic",
        vec![F64, F64, F64, F64],
        vec![F64],
        |builder, params| {
            // f(x) = ax² + bx + c
            let x = builder.use_var(params[0]);
            let a = builder.use_var(params[1]);
            let b = builder.use_var(params[2]);
            let c = builder.use_var(params[3]);

            let x_squared = builder.ins().fmul(x, x);
            let ax_squared = builder.ins().fmul(a, x_squared);
            let bx = builder.ins().fmul(b, x);
            let ax_squared_plus_bx = builder.ins().fadd(ax_squared, bx);
            let result = builder.ins().fadd(ax_squared_plus_bx, c);

            builder.ins().return_(&[result]);
        },
    )
}

/// Example: Using external function calls
pub fn compile_with_print(jit: &mut JitCompiler) -> Result<FuncId, String> {
    // First declare the external function
    let mut sig = jit.module.make_signature();
    sig.params.push(AbiParam::new(I64));

    let println_id = jit
        .module
        .declare_function("println_i64", Linkage::Import, &sig)
        .unwrap();

    // Define the function
    let func_id = jit
        .module
        .declare_function(
            "print_sum",
            Linkage::Export,
            &jit.make_signature(vec![I64, I64], vec![]),
        )
        .unwrap();

    // Create function context
    jit.ctx.func = Function::with_name_signature(
        UserFuncName::user(0, 0),
        jit.make_signature(vec![I64, I64], vec![]),
    );

    // Build the function
    {
        let mut builder = FunctionBuilder::new(&mut jit.ctx.func, &mut jit.builder_context);

        let entry_block = builder.create_block();
        builder.append_block_params_for_function_params(entry_block);
        builder.switch_to_block(entry_block);
        builder.seal_block(entry_block);

        let x = builder.declare_var(I64);
        let y = builder.declare_var(I64);

        let x_val = builder.block_params(entry_block)[0];
        let y_val = builder.block_params(entry_block)[1];
        builder.def_var(x, x_val);
        builder.def_var(y, y_val);

        let x_use = builder.use_var(x);
        let y_use = builder.use_var(y);
        let sum = builder.ins().iadd(x_use, y_use);

        // Declare the function reference for calling
        let println_ref = jit.module.declare_func_in_func(println_id, builder.func);
        builder.ins().call(println_ref, &[sum]);

        builder.ins().return_(&[]);
        builder.finalize();
    }

    // Verify the function
    if let Err(errors) = verify_function(&jit.ctx.func, jit.module.isa()) {
        return Err(format!("Function verification failed: {}", errors));
    }

    jit.module
        .define_function(func_id, &mut jit.ctx)
        .map_err(|e| e.to_string())?;

    jit.module.clear_context(&mut jit.ctx);

    Ok(func_id)
}

/// Example: Control flow with multiple returns
pub fn compile_max(jit: &mut JitCompiler) -> Result<FuncId, String> {
    jit.compile_function("max", vec![I64, I64], vec![I64], |builder, params| {
        let x = builder.use_var(params[0]);
        let y = builder.use_var(params[1]);

        let then_block = builder.create_block();
        let else_block = builder.create_block();

        // if x > y
        let cmp = builder.ins().icmp(IntCC::SignedGreaterThan, x, y);
        builder.ins().brif(cmp, then_block, &[], else_block, &[]);

        // then: return x
        builder.switch_to_block(then_block);
        builder.seal_block(then_block);
        builder.ins().return_(&[x]);

        // else: return y
        builder.switch_to_block(else_block);
        builder.seal_block(else_block);
        builder.ins().return_(&[y]);
    })
}

/// Example: Array/memory operations
pub fn compile_sum_array(jit: &mut JitCompiler) -> Result<FuncId, String> {
    jit.compile_function(
        "sum_array",
        vec![I64, I64], // ptr, len
        vec![I64],
        |builder, params| {
            let ptr = params[0];
            let len = params[1];

            // Create blocks
            let header_block = builder.create_block();
            let body_block = builder.create_block();
            let exit_block = builder.create_block();

            // Block parameters
            builder.append_block_param(header_block, I64); // index
            builder.append_block_param(header_block, I64); // sum
            builder.append_block_param(header_block, I64); // current_ptr

            // Initialize loop
            let zero = builder.ins().iconst(I64, 0);
            let ptr_val = builder.use_var(ptr);
            builder
                .ins()
                .jump(header_block, &[zero.into(), zero.into(), ptr_val.into()]);

            // Header: check if index < len
            builder.switch_to_block(header_block);
            let index = builder.block_params(header_block)[0];
            let sum = builder.block_params(header_block)[1];
            let current_ptr = builder.block_params(header_block)[2];
            let len_val = builder.use_var(len);
            let cmp = builder.ins().icmp(IntCC::UnsignedLessThan, index, len_val);
            builder.ins().brif(cmp, body_block, &[], exit_block, &[]);

            // Body: load value and add to sum
            builder.switch_to_block(body_block);
            builder.seal_block(body_block);
            let flags = MemFlags::new();
            let value = builder.ins().load(I64, flags, current_ptr, 0);
            let new_sum = builder.ins().iadd(sum, value);
            let new_index = builder.ins().iadd_imm(index, 1);
            let new_ptr = builder.ins().iadd_imm(current_ptr, 8); // 8 bytes for i64
            builder.ins().jump(
                header_block,
                &[new_index.into(), new_sum.into(), new_ptr.into()],
            );

            // Exit: return sum
            builder.switch_to_block(exit_block);
            builder.seal_block(exit_block);
            builder.seal_block(header_block);
            builder.ins().return_(&[sum]);
        },
    )
}

/// Example: Compile a simple expression evaluator
#[derive(Debug, Clone)]
pub enum Expr {
    Const(i64),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Var(usize),
}

impl Expr {
    pub fn compile(&self, builder: &mut FunctionBuilder, vars: &[Variable]) -> Value {
        match self {
            Expr::Const(n) => builder.ins().iconst(I64, *n),
            Expr::Add(a, b) => {
                let a_val = a.compile(builder, vars);
                let b_val = b.compile(builder, vars);
                builder.ins().iadd(a_val, b_val)
            }
            Expr::Sub(a, b) => {
                let a_val = a.compile(builder, vars);
                let b_val = b.compile(builder, vars);
                builder.ins().isub(a_val, b_val)
            }
            Expr::Mul(a, b) => {
                let a_val = a.compile(builder, vars);
                let b_val = b.compile(builder, vars);
                builder.ins().imul(a_val, b_val)
            }
            Expr::Var(idx) => builder.use_var(vars[*idx]),
        }
    }
}

pub fn compile_expression(jit: &mut JitCompiler, expr: Expr) -> Result<FuncId, String> {
    jit.compile_function(
        "eval_expr",
        vec![I64, I64], // two variables
        vec![I64],
        |builder, params| {
            let result = expr.compile(builder, params);
            builder.ins().return_(&[result]);
        },
    )
}

/// Symbol table for variable management
pub struct SymbolTable {
    variables: HashMap<String, Variable>,
    next_var: usize,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            next_var: 0,
        }
    }

    pub fn declare(&mut self, name: String, builder: &mut FunctionBuilder, ty: Type) -> Variable {
        let var = builder.declare_var(ty);
        self.variables.insert(name.clone(), var);
        self.next_var += 1;
        var
    }

    pub fn get(&self, name: &str) -> Option<Variable> {
        self.variables.get(name).copied()
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_add() {
        let mut jit = JitCompiler::new();
        let func_id = compile_add_function(&mut jit).unwrap();
        jit.finalize();

        let code = jit.get_function(func_id);
        let add_fn = unsafe { std::mem::transmute::<*const u8, fn(i64, i64) -> i64>(code) };

        assert_eq!(add_fn(2, 3), 5);
        assert_eq!(add_fn(10, -5), 5);
    }

    #[test]
    fn test_compile_factorial() {
        let mut jit = JitCompiler::new();
        let func_id = compile_factorial(&mut jit).unwrap();
        jit.finalize();

        let code = jit.get_function(func_id);
        let factorial_fn = unsafe { std::mem::transmute::<*const u8, fn(i64) -> i64>(code) };

        assert_eq!(factorial_fn(0), 1);
        assert_eq!(factorial_fn(1), 1);
        assert_eq!(factorial_fn(5), 120);
    }

    #[test]
    fn test_compile_max() {
        let mut jit = JitCompiler::new();
        let func_id = compile_max(&mut jit).unwrap();
        jit.finalize();

        let code = jit.get_function(func_id);
        let max_fn = unsafe { std::mem::transmute::<*const u8, fn(i64, i64) -> i64>(code) };

        assert_eq!(max_fn(5, 3), 5);
        assert_eq!(max_fn(2, 8), 8);
        assert_eq!(max_fn(-5, -3), -3);
    }

    #[test]
    fn test_compile_expression() {
        let mut jit = JitCompiler::new();

        // (x + 3) * (y - 2)
        let expr = Expr::Mul(
            Box::new(Expr::Add(Box::new(Expr::Var(0)), Box::new(Expr::Const(3)))),
            Box::new(Expr::Sub(Box::new(Expr::Var(1)), Box::new(Expr::Const(2)))),
        );

        let func_id = compile_expression(&mut jit, expr).unwrap();
        jit.finalize();

        let code = jit.get_function(func_id);
        let eval_fn = unsafe { std::mem::transmute::<*const u8, fn(i64, i64) -> i64>(code) };

        assert_eq!(eval_fn(5, 7), 40); // (5+3) * (7-2) = 8 * 5 = 40
        assert_eq!(eval_fn(2, 4), 10); // (2+3) * (4-2) = 5 * 2 = 10
    }

    #[test]
    fn test_quadratic() {
        let mut jit = JitCompiler::new();
        let func_id = compile_quadratic(&mut jit).unwrap();
        jit.finalize();

        let code = jit.get_function(func_id);
        let quad_fn =
            unsafe { std::mem::transmute::<*const u8, fn(f64, f64, f64, f64) -> f64>(code) };

        // f(x) = 2x² + 3x + 1
        // f(2) = 2*4 + 3*2 + 1 = 8 + 6 + 1 = 15
        assert_eq!(quad_fn(2.0, 2.0, 3.0, 1.0), 15.0);
    }
}
