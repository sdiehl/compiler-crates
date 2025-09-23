use std::error::Error;
use std::path::Path;

use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::passes::PassBuilderOptions;
use inkwell::targets::{
    CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine,
};
use inkwell::types::{BasicMetadataTypeEnum, BasicType, BasicTypeEnum};
use inkwell::values::FunctionValue;
use inkwell::{AddressSpace, IntPredicate, OptimizationLevel};

/// Creates a basic LLVM context
pub fn create_context() -> Context {
    Context::create()
}

/// Creates a simple function that adds two integers
pub fn create_add_function<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
) -> FunctionValue<'ctx> {
    let i32_type = context.i32_type();
    let fn_type = i32_type.fn_type(&[i32_type.into(), i32_type.into()], false);
    let function = module.add_function("add", fn_type, None);

    let builder = context.create_builder();
    let entry = context.append_basic_block(function, "entry");
    builder.position_at_end(entry);

    // Get function parameters
    let x = function.get_nth_param(0).unwrap().into_int_value();
    let y = function.get_nth_param(1).unwrap().into_int_value();

    // Build addition and return
    let sum = builder.build_int_add(x, y, "sum").unwrap();
    builder.build_return(Some(&sum)).unwrap();

    function
}

/// Creates a function that multiplies two integers
pub fn create_multiply_function<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
) -> FunctionValue<'ctx> {
    let i32_type = context.i32_type();
    let fn_type = i32_type.fn_type(&[i32_type.into(), i32_type.into()], false);
    let function = module.add_function("multiply", fn_type, None);

    let builder = context.create_builder();
    let entry = context.append_basic_block(function, "entry");
    builder.position_at_end(entry);

    let x = function.get_nth_param(0).unwrap().into_int_value();
    let y = function.get_nth_param(1).unwrap().into_int_value();

    let product = builder.build_int_mul(x, y, "product").unwrap();
    builder.build_return(Some(&product)).unwrap();

    function
}

/// Creates a function with a constant return value
pub fn create_constant_function<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
) -> FunctionValue<'ctx> {
    let i32_type = context.i32_type();
    let fn_type = i32_type.fn_type(&[], false);
    let function = module.add_function("get_constant", fn_type, None);

    let builder = context.create_builder();
    let entry = context.append_basic_block(function, "entry");
    builder.position_at_end(entry);

    let constant = i32_type.const_int(42, false);
    builder.build_return(Some(&constant)).unwrap();

    function
}

/// Demonstrates integer comparison operations
pub fn create_comparison_function<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
) -> FunctionValue<'ctx> {
    let i32_type = context.i32_type();
    let bool_type = context.bool_type();
    let fn_type = bool_type.fn_type(&[i32_type.into(), i32_type.into()], false);
    let function = module.add_function("compare_ints", fn_type, None);

    let builder = context.create_builder();
    let entry = context.append_basic_block(function, "entry");
    builder.position_at_end(entry);

    let x = function.get_nth_param(0).unwrap().into_int_value();
    let y = function.get_nth_param(1).unwrap().into_int_value();

    // Compare x > y
    let comparison = builder
        .build_int_compare(IntPredicate::SGT, x, y, "cmp")
        .unwrap();
    builder.build_return(Some(&comparison)).unwrap();

    function
}

/// Creates a function with conditional branching (if-then-else)
pub fn create_conditional_function<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
) -> FunctionValue<'ctx> {
    let i32_type = context.i32_type();
    let fn_type = i32_type.fn_type(&[i32_type.into()], false);
    let function = module.add_function("conditional", fn_type, None);

    let builder = context.create_builder();
    let entry = context.append_basic_block(function, "entry");
    let then_block = context.append_basic_block(function, "then");
    let else_block = context.append_basic_block(function, "else");
    let merge_block = context.append_basic_block(function, "merge");

    // Entry block
    builder.position_at_end(entry);
    let x = function.get_nth_param(0).unwrap().into_int_value();
    let zero = i32_type.const_int(0, false);
    let condition = builder
        .build_int_compare(IntPredicate::SGT, x, zero, "cond")
        .unwrap();
    builder
        .build_conditional_branch(condition, then_block, else_block)
        .unwrap();

    // Then block: return x * 2
    builder.position_at_end(then_block);
    let two = i32_type.const_int(2, false);
    let then_val = builder.build_int_mul(x, two, "then_val").unwrap();
    builder.build_unconditional_branch(merge_block).unwrap();

    // Else block: return x * -1
    builder.position_at_end(else_block);
    let neg_one = i32_type.const_int(-1i64 as u64, true);
    let else_val = builder.build_int_mul(x, neg_one, "else_val").unwrap();
    builder.build_unconditional_branch(merge_block).unwrap();

    // Merge block with phi node
    builder.position_at_end(merge_block);
    let phi = builder.build_phi(i32_type, "result").unwrap();
    phi.add_incoming(&[(&then_val, then_block), (&else_val, else_block)]);
    builder.build_return(Some(&phi.as_basic_value())).unwrap();

    function
}

/// Creates a simple loop that counts from 0 to n
pub fn create_loop_function<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
) -> FunctionValue<'ctx> {
    let i32_type = context.i32_type();
    let fn_type = i32_type.fn_type(&[i32_type.into()], false);
    let function = module.add_function("count_loop", fn_type, None);

    let builder = context.create_builder();
    let entry = context.append_basic_block(function, "entry");
    let loop_block = context.append_basic_block(function, "loop");
    let exit_block = context.append_basic_block(function, "exit");

    // Entry block: initialize counter
    builder.position_at_end(entry);
    let n = function.get_nth_param(0).unwrap().into_int_value();
    let zero = i32_type.const_int(0, false);
    builder.build_unconditional_branch(loop_block).unwrap();

    // Loop block
    builder.position_at_end(loop_block);
    let counter = builder.build_phi(i32_type, "counter").unwrap();
    counter.add_incoming(&[(&zero, entry)]);

    // Increment counter
    let one = i32_type.const_int(1, false);
    let next_counter = builder
        .build_int_add(
            counter.as_basic_value().into_int_value(),
            one,
            "next_counter",
        )
        .unwrap();

    // Check loop condition
    let condition = builder
        .build_int_compare(IntPredicate::SLT, next_counter, n, "loop_cond")
        .unwrap();

    // Add incoming value for next iteration
    let loop_block_end = builder.get_insert_block().unwrap();
    counter.add_incoming(&[(&next_counter, loop_block_end)]);

    builder
        .build_conditional_branch(condition, loop_block, exit_block)
        .unwrap();

    // Exit block
    builder.position_at_end(exit_block);
    builder
        .build_return(Some(&counter.as_basic_value()))
        .unwrap();

    function
}

/// Creates a function that allocates and uses local variables (stack
/// allocation)
pub fn create_alloca_function<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
) -> FunctionValue<'ctx> {
    let i32_type = context.i32_type();
    let fn_type = i32_type.fn_type(&[i32_type.into(), i32_type.into()], false);
    let function = module.add_function("use_alloca", fn_type, None);

    let builder = context.create_builder();
    let entry = context.append_basic_block(function, "entry");
    builder.position_at_end(entry);

    // Get parameters
    let x = function.get_nth_param(0).unwrap().into_int_value();
    let y = function.get_nth_param(1).unwrap().into_int_value();

    // Allocate stack variables
    let x_ptr = builder.build_alloca(i32_type, "x_ptr").unwrap();
    let y_ptr = builder.build_alloca(i32_type, "y_ptr").unwrap();
    let result_ptr = builder.build_alloca(i32_type, "result_ptr").unwrap();

    // Store values
    builder.build_store(x_ptr, x).unwrap();
    builder.build_store(y_ptr, y).unwrap();

    // Load values
    let x_val = builder.build_load(i32_type, x_ptr, "x_val").unwrap();
    let y_val = builder.build_load(i32_type, y_ptr, "y_val").unwrap();

    // Compute and store result
    let sum = builder
        .build_int_add(x_val.into_int_value(), y_val.into_int_value(), "sum")
        .unwrap();
    builder.build_store(result_ptr, sum).unwrap();

    // Load and return result
    let result = builder.build_load(i32_type, result_ptr, "result").unwrap();
    builder.build_return(Some(&result)).unwrap();

    function
}

/// Creates a function that works with arrays
pub fn create_array_function<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
) -> FunctionValue<'ctx> {
    let i32_type = context.i32_type();
    let i32_ptr_type = context.ptr_type(AddressSpace::default());
    let fn_type = i32_type.fn_type(&[i32_ptr_type.into(), i32_type.into()], false);
    let function = module.add_function("sum_array", fn_type, None);

    let builder = context.create_builder();
    let entry = context.append_basic_block(function, "entry");
    let loop_block = context.append_basic_block(function, "loop");
    let exit_block = context.append_basic_block(function, "exit");

    // Entry block
    builder.position_at_end(entry);
    let array_ptr = function.get_nth_param(0).unwrap().into_pointer_value();
    let size = function.get_nth_param(1).unwrap().into_int_value();
    let zero = i32_type.const_int(0, false);
    builder.build_unconditional_branch(loop_block).unwrap();

    // Loop block
    builder.position_at_end(loop_block);
    let index = builder.build_phi(i32_type, "index").unwrap();
    let sum = builder.build_phi(i32_type, "sum").unwrap();
    index.add_incoming(&[(&zero, entry)]);
    sum.add_incoming(&[(&zero, entry)]);

    // Load array element
    let elem_ptr = unsafe {
        builder
            .build_gep(
                i32_type,
                array_ptr,
                &[index.as_basic_value().into_int_value()],
                "elem_ptr",
            )
            .unwrap()
    };
    let elem = builder.build_load(i32_type, elem_ptr, "elem").unwrap();

    // Update sum
    let new_sum = builder
        .build_int_add(
            sum.as_basic_value().into_int_value(),
            elem.into_int_value(),
            "new_sum",
        )
        .unwrap();

    // Update index
    let one = i32_type.const_int(1, false);
    let next_index = builder
        .build_int_add(index.as_basic_value().into_int_value(), one, "next_index")
        .unwrap();

    // Check loop condition
    let condition = builder
        .build_int_compare(IntPredicate::SLT, next_index, size, "loop_cond")
        .unwrap();

    // Update phi nodes
    let loop_end = builder.get_insert_block().unwrap();
    index.add_incoming(&[(&next_index, loop_end)]);
    sum.add_incoming(&[(&new_sum, loop_end)]);

    builder
        .build_conditional_branch(condition, loop_block, exit_block)
        .unwrap();

    // Exit block
    builder.position_at_end(exit_block);
    builder.build_return(Some(&sum.as_basic_value())).unwrap();

    function
}

/// Creates a global variable
pub fn create_global_variable<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
) -> inkwell::values::GlobalValue<'ctx> {
    let i32_type = context.i32_type();
    let global = module.add_global(i32_type, Some(AddressSpace::default()), "global_counter");
    global.set_initializer(&i32_type.const_int(0, false));
    global.set_linkage(inkwell::module::Linkage::Internal);
    global
}

/// Creates a function that uses a global variable
pub fn create_global_function<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
) -> FunctionValue<'ctx> {
    let i32_type = context.i32_type();
    let fn_type = i32_type.fn_type(&[], false);
    let function = module.add_function("increment_global", fn_type, None);

    // Create or get global variable
    let global = module
        .get_global("global_counter")
        .unwrap_or_else(|| create_global_variable(context, module));

    let builder = context.create_builder();
    let entry = context.append_basic_block(function, "entry");
    builder.position_at_end(entry);

    // Load global value
    let global_ptr = global.as_pointer_value();
    let current = builder.build_load(i32_type, global_ptr, "current").unwrap();

    // Increment
    let one = i32_type.const_int(1, false);
    let incremented = builder
        .build_int_add(current.into_int_value(), one, "incremented")
        .unwrap();

    // Store back to global
    builder.build_store(global_ptr, incremented).unwrap();

    // Return new value
    builder.build_return(Some(&incremented)).unwrap();

    function
}

/// Creates a recursive factorial function
pub fn create_recursive_function<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
) -> FunctionValue<'ctx> {
    let i32_type = context.i32_type();
    let fn_type = i32_type.fn_type(&[i32_type.into()], false);
    let function = module.add_function("factorial", fn_type, None);

    let builder = context.create_builder();
    let entry = context.append_basic_block(function, "entry");
    let recurse = context.append_basic_block(function, "recurse");
    let base = context.append_basic_block(function, "base");

    // Entry block
    builder.position_at_end(entry);
    let n = function.get_nth_param(0).unwrap().into_int_value();
    let one = i32_type.const_int(1, false);
    let is_base = builder
        .build_int_compare(IntPredicate::SLE, n, one, "is_base")
        .unwrap();
    builder
        .build_conditional_branch(is_base, base, recurse)
        .unwrap();

    // Base case: return 1
    builder.position_at_end(base);
    builder.build_return(Some(&one)).unwrap();

    // Recursive case: return n * factorial(n-1)
    builder.position_at_end(recurse);
    let n_minus_1 = builder.build_int_sub(n, one, "n_minus_1").unwrap();
    let rec_result = builder
        .build_call(function, &[n_minus_1.into()], "rec_result")
        .unwrap();
    let result = builder
        .build_int_mul(
            n,
            rec_result
                .try_as_basic_value()
                .left()
                .unwrap()
                .into_int_value(),
            "result",
        )
        .unwrap();
    builder.build_return(Some(&result)).unwrap();

    function
}

/// Creates a struct type and a function that uses it
pub fn create_struct_function<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
) -> FunctionValue<'ctx> {
    let i32_type = context.i32_type();
    let f64_type = context.f64_type();

    // Define a Point struct with x and y fields
    let field_types = [i32_type.into(), f64_type.into()];
    let struct_type = context.struct_type(&field_types, false);

    let fn_type = f64_type.fn_type(&[struct_type.into()], false);
    let function = module.add_function("get_point_y", fn_type, None);

    let builder = context.create_builder();
    let entry = context.append_basic_block(function, "entry");
    builder.position_at_end(entry);

    // Get the struct parameter
    let point = function.get_nth_param(0).unwrap().into_struct_value();

    // Extract the y field (index 1)
    let y_field = builder.build_extract_value(point, 1, "y_field").unwrap();

    builder.build_return(Some(&y_field)).unwrap();

    function
}

/// Runs optimization passes on a module using the modern pass manager (LLVM 18)
pub fn optimize_module<'ctx>(module: &Module<'ctx>) -> Result<(), String> {
    // First verify the module is valid
    module.verify().map_err(|e| e.to_string())?;

    // Initialize targets for optimization
    Target::initialize_all(&InitializationConfig::default());

    let target_triple = TargetMachine::get_default_triple();
    let target = Target::from_triple(&target_triple)
        .map_err(|e| format!("Failed to create target: {}", e))?;

    let target_machine = target
        .create_target_machine(
            &target_triple,
            "generic",
            "",
            OptimizationLevel::Default,
            RelocMode::Default,
            CodeModel::Default,
        )
        .ok_or("Failed to create target machine")?;

    // Common optimization passes
    let passes = [
        "instcombine", // Combine instructions
        "reassociate", // Reassociate expressions
        "gvn",         // Global value numbering
        "simplifycfg", // Simplify control flow graph
        "mem2reg",     // Promote memory to registers
    ];

    let pass_builder_options = PassBuilderOptions::create();
    pass_builder_options.set_loop_vectorization(true);
    pass_builder_options.set_loop_unrolling(true);
    pass_builder_options.set_merge_functions(true);

    module
        .run_passes(&passes.join(","), &target_machine, pass_builder_options)
        .map_err(|e| e.to_string())
}

/// Runs specific optimization passes on a module
pub fn run_custom_passes<'ctx>(module: &Module<'ctx>, passes: &[&str]) -> Result<(), String> {
    // Verify module first
    module.verify().map_err(|e| e.to_string())?;

    // Initialize targets
    Target::initialize_all(&InitializationConfig::default());

    let target_triple = TargetMachine::get_default_triple();
    let target = Target::from_triple(&target_triple)
        .map_err(|e| format!("Failed to create target: {}", e))?;

    let target_machine = target
        .create_target_machine(
            &target_triple,
            "generic",
            "",
            OptimizationLevel::None,
            RelocMode::Default,
            CodeModel::Default,
        )
        .ok_or("Failed to create target machine")?;

    module
        .run_passes(
            &passes.join(","),
            &target_machine,
            PassBuilderOptions::create(),
        )
        .map_err(|e| e.to_string())
}

/// Writes LLVM IR to a file
pub fn write_ir_to_file<'ctx>(module: &Module<'ctx>, path: &Path) -> Result<(), String> {
    module
        .print_to_file(path)
        .map_err(|e| format!("Failed to write IR: {}", e))
}

/// Compiles module to object file
pub fn compile_to_object_file<'ctx>(
    module: &Module<'ctx>,
    path: &Path,
) -> Result<(), Box<dyn Error>> {
    Target::initialize_native(&InitializationConfig::default())?;

    let target_triple = TargetMachine::get_default_triple();
    let target = Target::from_triple(&target_triple)?;

    let target_machine = target
        .create_target_machine(
            &target_triple,
            "generic",
            "",
            OptimizationLevel::Default,
            RelocMode::Default,
            CodeModel::Default,
        )
        .ok_or("Could not create target machine")?;

    target_machine.write_to_file(module, FileType::Object, path)?;
    Ok(())
}

/// Verifies that a module is valid
pub fn verify_module<'ctx>(module: &Module<'ctx>) -> Result<(), String> {
    module.verify().map_err(|e| e.to_string())
}

/// Helper to create a function type
pub fn create_function_type<'ctx>(
    context: &'ctx Context,
    param_types: Vec<BasicMetadataTypeEnum<'ctx>>,
    return_type: Option<BasicTypeEnum<'ctx>>,
    is_var_args: bool,
) -> inkwell::types::FunctionType<'ctx> {
    match return_type {
        Some(ret) => ret.fn_type(&param_types, is_var_args),
        None => context.void_type().fn_type(&param_types, is_var_args),
    }
}

/// Simple JIT execution example
pub fn create_execution_engine<'ctx>(
    module: &Module<'ctx>,
) -> Result<inkwell::execution_engine::ExecutionEngine<'ctx>, String> {
    module
        .create_jit_execution_engine(OptimizationLevel::None)
        .map_err(|e| e.to_string())
}

/// Example of JIT compiling and executing a function
pub fn jit_compile_and_execute(context: &Context) -> Result<u64, Box<dyn Error>> {
    let module = context.create_module("jit_example");
    let builder = context.create_builder();

    // Create a simple sum function: sum(x, y, z) = x + y + z
    let i64_type = context.i64_type();
    let fn_type = i64_type.fn_type(&[i64_type.into(), i64_type.into(), i64_type.into()], false);
    let function = module.add_function("sum", fn_type, None);
    let basic_block = context.append_basic_block(function, "entry");

    builder.position_at_end(basic_block);

    let x = function.get_nth_param(0).unwrap().into_int_value();
    let y = function.get_nth_param(1).unwrap().into_int_value();
    let z = function.get_nth_param(2).unwrap().into_int_value();

    let sum = builder.build_int_add(x, y, "sum").unwrap();
    let sum = builder.build_int_add(sum, z, "sum").unwrap();
    builder.build_return(Some(&sum)).unwrap();

    // Create execution engine
    let execution_engine = module.create_jit_execution_engine(OptimizationLevel::None)?;

    // Get the compiled function
    type SumFunc = unsafe extern "C" fn(u64, u64, u64) -> u64;
    let sum_fn = unsafe { execution_engine.get_function::<SumFunc>("sum")? };

    // Execute the function
    let result = unsafe { sum_fn.call(1, 2, 3) };

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_function() {
        let context = Context::create();
        let module = context.create_module("test");
        let function = create_add_function(&context, &module);

        assert_eq!(function.get_name().to_str().unwrap(), "add");
        assert_eq!(function.count_params(), 2);
        assert!(verify_module(&module).is_ok());
    }

    #[test]
    fn test_constant_function() {
        let context = Context::create();
        let module = context.create_module("test");
        let function = create_constant_function(&context, &module);

        assert_eq!(function.get_name().to_str().unwrap(), "get_constant");
        assert_eq!(function.count_params(), 0);
        assert!(verify_module(&module).is_ok());
    }

    #[test]
    fn test_conditional_function() {
        let context = Context::create();
        let module = context.create_module("test");
        let function = create_conditional_function(&context, &module);

        assert_eq!(function.count_basic_blocks(), 4); // entry, then, else, merge
        assert!(verify_module(&module).is_ok());
    }

    #[test]
    fn test_loop_function() {
        let context = Context::create();
        let module = context.create_module("test");
        let function = create_loop_function(&context, &module);

        assert_eq!(function.count_basic_blocks(), 3); // entry, loop, exit
        assert!(verify_module(&module).is_ok());
    }

    #[test]
    fn test_global_variable() {
        let context = Context::create();
        let module = context.create_module("test");
        let global = create_global_variable(&context, &module);

        assert_eq!(global.get_name().to_str().unwrap(), "global_counter");
        assert!(verify_module(&module).is_ok());
    }

    #[test]
    fn test_recursive_function() {
        let context = Context::create();
        let module = context.create_module("test");
        let function = create_recursive_function(&context, &module);

        assert_eq!(function.get_name().to_str().unwrap(), "factorial");
        assert!(verify_module(&module).is_ok());
    }

    #[test]
    fn test_optimization() {
        let context = Context::create();
        let module = context.create_module("test");

        // Create several functions
        create_add_function(&context, &module);
        create_multiply_function(&context, &module);
        create_constant_function(&context, &module);

        // Apply optimizations
        assert!(optimize_module(&module).is_ok());

        // Module should still be valid after optimization
        assert!(verify_module(&module).is_ok());
    }

    #[test]
    fn test_custom_passes() {
        let context = Context::create();
        let module = context.create_module("test");

        // Create a simple function
        create_add_function(&context, &module);

        // Run specific optimization passes
        let passes = ["instcombine", "simplifycfg"];
        assert!(run_custom_passes(&module, &passes).is_ok());

        // Module should still be valid
        assert!(verify_module(&module).is_ok());
    }

    #[test]
    fn test_jit_execution() {
        let context = Context::create();

        // Test JIT compilation and execution
        match jit_compile_and_execute(&context) {
            Ok(result) => assert_eq!(result, 6), // 1 + 2 + 3 = 6
            Err(e) => panic!("JIT execution failed: {}", e),
        }
    }
}
