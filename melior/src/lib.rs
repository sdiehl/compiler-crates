use melior::dialect::{arith, func, DialectRegistry};
use melior::ir::attribute::{IntegerAttribute, StringAttribute, TypeAttribute};
use melior::ir::operation::OperationLike;
use melior::ir::r#type::{FunctionType, IntegerType};
use melior::ir::*;
use melior::pass::{gpu, transform, PassManager};
use melior::utility::{register_all_dialects, register_all_llvm_translations, register_all_passes};
use melior::{Context, Error};

/// Creates a test context with all dialects loaded
pub fn create_test_context() -> Context {
    let context = Context::new();
    let registry = DialectRegistry::new();
    register_all_dialects(&registry);
    register_all_passes();

    context.append_dialect_registry(&registry);
    context.load_all_available_dialects();
    register_all_llvm_translations(&context);

    context
}

/// Creates a simple function that adds two integers
pub fn create_add_function(context: &Context) -> Result<Module<'_>, Error> {
    let location = Location::unknown(context);
    let module = Module::new(location);
    let index_type = Type::index(context);

    module.body().append_operation(func::func(
        context,
        StringAttribute::new(context, "add"),
        TypeAttribute::new(
            FunctionType::new(context, &[index_type, index_type], &[index_type]).into(),
        ),
        {
            let block = Block::new(&[(index_type, location), (index_type, location)]);

            let sum = block
                .append_operation(arith::addi(
                    block.argument(0).unwrap().into(),
                    block.argument(1).unwrap().into(),
                    location,
                ))
                .result(0)
                .unwrap();

            block.append_operation(func::r#return(&[sum.into()], location));

            let region = Region::new();
            region.append_block(block);
            region
        },
        &[],
        location,
    ));

    Ok(module)
}

/// Creates a function that multiplies two 32-bit integers
pub fn create_multiply_function(context: &Context) -> Result<Module<'_>, Error> {
    let location = Location::unknown(context);
    let module = Module::new(location);
    let i32_type = IntegerType::new(context, 32).into();

    module.body().append_operation(func::func(
        context,
        StringAttribute::new(context, "multiply"),
        TypeAttribute::new(FunctionType::new(context, &[i32_type, i32_type], &[i32_type]).into()),
        {
            let block = Block::new(&[(i32_type, location), (i32_type, location)]);

            let product = block
                .append_operation(arith::muli(
                    block.argument(0).unwrap().into(),
                    block.argument(1).unwrap().into(),
                    location,
                ))
                .result(0)
                .unwrap();

            block.append_operation(func::r#return(&[product.into()], location));

            let region = Region::new();
            region.append_block(block);
            region
        },
        &[],
        location,
    ));

    Ok(module)
}

/// Creates a constant integer value
pub fn create_constant(context: &Context, value: i64) -> Result<Module<'_>, Error> {
    let location = Location::unknown(context);
    let module = Module::new(location);
    let i64_type = IntegerType::new(context, 64).into();

    module.body().append_operation(func::func(
        context,
        StringAttribute::new(context, "get_constant"),
        TypeAttribute::new(FunctionType::new(context, &[], &[i64_type]).into()),
        {
            let block = Block::new(&[]);

            let constant = block
                .append_operation(arith::constant(
                    context,
                    IntegerAttribute::new(i64_type, value).into(),
                    location,
                ))
                .result(0)
                .unwrap();

            block.append_operation(func::r#return(&[constant.into()], location));

            let region = Region::new();
            region.append_block(block);
            region
        },
        &[],
        location,
    ));

    Ok(module)
}

/// Shows how to verify MLIR modules
pub fn verify_module(module: &Module<'_>) -> bool {
    module.as_operation().verify()
}

/// Shows how to print MLIR to string
pub fn module_to_string(module: &Module<'_>) -> String {
    format!("{}", module.as_operation())
}

/// Apply canonicalization transforms to simplify IR
pub fn apply_canonicalization(context: &Context, module: &mut Module<'_>) -> Result<(), Error> {
    let pass_manager = PassManager::new(context);
    pass_manager.add_pass(transform::create_canonicalizer());
    pass_manager.run(module)
}

/// Apply CSE (Common Subexpression Elimination) to remove redundant
/// computations
pub fn apply_cse(context: &Context, module: &mut Module<'_>) -> Result<(), Error> {
    let pass_manager = PassManager::new(context);
    pass_manager.add_pass(transform::create_cse());
    pass_manager.run(module)
}

/// Apply inlining transformation to inline function calls
pub fn apply_inlining(context: &Context, module: &mut Module<'_>) -> Result<(), Error> {
    let pass_manager = PassManager::new(context);
    pass_manager.add_pass(transform::create_inliner());
    pass_manager.run(module)
}

/// Apply loop-invariant code motion to optimize loops
pub fn apply_licm(context: &Context, module: &mut Module<'_>) -> Result<(), Error> {
    let pass_manager = PassManager::new(context);
    pass_manager.add_pass(transform::create_loop_invariant_code_motion());
    pass_manager.run(module)
}

/// Apply SCCP (Sparse Conditional Constant Propagation) for constant folding
pub fn apply_sccp(context: &Context, module: &mut Module<'_>) -> Result<(), Error> {
    let pass_manager = PassManager::new(context);
    pass_manager.add_pass(transform::create_sccp());
    pass_manager.run(module)
}

/// Apply a pipeline of optimization passes
pub fn optimize_module(context: &Context, module: &mut Module<'_>) -> Result<(), Error> {
    let pass_manager = PassManager::new(context);

    // Standard optimization pipeline
    pass_manager.add_pass(transform::create_canonicalizer());
    pass_manager.add_pass(transform::create_cse());
    pass_manager.add_pass(transform::create_sccp());
    pass_manager.add_pass(transform::create_inliner());
    pass_manager.add_pass(transform::create_canonicalizer()); // Run again after inlining

    pass_manager.run(module)
}

/// Example of applying symbol DCE (Dead Code Elimination)
pub fn apply_symbol_dce(context: &Context, module: &mut Module<'_>) -> Result<(), Error> {
    let pass_manager = PassManager::new(context);
    pass_manager.add_pass(transform::create_symbol_dce());
    pass_manager.run(module)
}

/// Apply mem2reg transformation to promote memory to registers
pub fn apply_mem2reg(context: &Context, module: &mut Module<'_>) -> Result<(), Error> {
    let pass_manager = PassManager::new(context);
    pass_manager.add_pass(transform::create_mem_2_reg());
    pass_manager.run(module)
}

/// Convert parallel loops to GPU kernels
pub fn convert_to_gpu(context: &Context, module: &mut Module<'_>) -> Result<(), Error> {
    let pass_manager = PassManager::new(context);
    pass_manager.add_pass(gpu::create_gpu_kernel_outlining());
    pass_manager.run(module)
}

/// Strip debug information from the module
pub fn strip_debug_info(context: &Context, module: &mut Module<'_>) -> Result<(), Error> {
    let pass_manager = PassManager::new(context);
    pass_manager.add_pass(transform::create_strip_debug_info());
    pass_manager.run(module)
}

/// Type builder helper for creating complex types
pub struct TypeBuilder<'c> {
    context: &'c Context,
}

impl<'c> TypeBuilder<'c> {
    pub fn new(context: &'c Context) -> Self {
        Self { context }
    }

    pub fn i32(&self) -> Type<'c> {
        IntegerType::new(self.context, 32).into()
    }

    pub fn i64(&self) -> Type<'c> {
        IntegerType::new(self.context, 64).into()
    }

    pub fn index(&self) -> Type<'c> {
        Type::index(self.context)
    }

    pub fn function(&self, inputs: &[Type<'c>], outputs: &[Type<'c>]) -> FunctionType<'c> {
        FunctionType::new(self.context, inputs, outputs)
    }
}

/// Attribute builder helper for creating attributes
pub struct AttributeBuilder<'c> {
    context: &'c Context,
}

impl<'c> AttributeBuilder<'c> {
    pub fn new(context: &'c Context) -> Self {
        Self { context }
    }

    pub fn string(&self, value: &str) -> StringAttribute<'c> {
        StringAttribute::new(self.context, value)
    }

    pub fn integer(&self, ty: Type<'c>, value: i64) -> IntegerAttribute<'c> {
        IntegerAttribute::new(ty, value)
    }

    pub fn type_attr(&self, ty: Type<'c>) -> TypeAttribute<'c> {
        TypeAttribute::new(ty)
    }
}

/// Custom pass builder for creating transformation pipelines
pub struct PassPipeline<'c> {
    pass_manager: PassManager<'c>,
}

impl<'c> PassPipeline<'c> {
    pub fn new(context: &'c Context) -> Self {
        Self {
            pass_manager: PassManager::new(context),
        }
    }

    /// Add a canonicalization pass
    pub fn canonicalize(self) -> Self {
        self.pass_manager
            .add_pass(transform::create_canonicalizer());
        self
    }

    /// Add a CSE pass
    pub fn eliminate_common_subexpressions(self) -> Self {
        self.pass_manager.add_pass(transform::create_cse());
        self
    }

    /// Add an inlining pass
    pub fn inline_functions(self) -> Self {
        self.pass_manager.add_pass(transform::create_inliner());
        self
    }

    /// Add SCCP for constant propagation
    pub fn propagate_constants(self) -> Self {
        self.pass_manager.add_pass(transform::create_sccp());
        self
    }

    /// Add loop optimizations
    pub fn optimize_loops(self) -> Self {
        self.pass_manager
            .add_pass(transform::create_loop_invariant_code_motion());
        self
    }

    /// Run the pipeline on a module
    pub fn run(self, module: &mut Module<'c>) -> Result<(), Error> {
        self.pass_manager.run(module)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_function() {
        let context = create_test_context();
        let module = create_add_function(&context).unwrap();
        assert!(verify_module(&module));
        let ir = module_to_string(&module);
        assert!(ir.contains("func.func @add"));
        assert!(ir.contains("arith.addi"));
    }

    #[test]
    fn test_multiply_function() {
        let context = create_test_context();
        let module = create_multiply_function(&context).unwrap();
        assert!(verify_module(&module));
        let ir = module_to_string(&module);
        assert!(ir.contains("func.func @multiply"));
        assert!(ir.contains("arith.muli"));
    }

    #[test]
    fn test_constant_creation() {
        let context = create_test_context();
        let module = create_constant(&context, 42).unwrap();
        assert!(verify_module(&module));
        let ir = module_to_string(&module);
        assert!(ir.contains("arith.constant"));
        assert!(ir.contains("42"));
    }

    #[test]
    fn test_type_builder() {
        let context = create_test_context();
        let types = TypeBuilder::new(&context);

        let i32 = types.i32();
        let i64 = types.i64();
        let func_type = types.function(&[i32, i32], &[i64]);

        assert_eq!(func_type.input(0).unwrap(), i32);
        assert_eq!(func_type.input(1).unwrap(), i32);
        assert_eq!(func_type.result(0).unwrap(), i64);
    }

    #[test]
    fn test_attribute_builder() {
        let context = create_test_context();
        let types = TypeBuilder::new(&context);
        let attrs = AttributeBuilder::new(&context);

        let name = attrs.string("test_function");
        // StringAttribute doesn't expose string value directly in the API
        assert!(name.is_string());

        let value = attrs.integer(types.i32(), 100);
        assert_eq!(value.value(), 100);
    }

    #[test]
    fn test_canonicalization() {
        let context = create_test_context();
        let mut module = create_add_function(&context).unwrap();

        let _before = module_to_string(&module);
        apply_canonicalization(&context, &mut module).unwrap();
        let after = module_to_string(&module);

        assert!(verify_module(&module));
        // Canonicalization should preserve or simplify the IR
        assert!(!after.is_empty());
    }

    #[test]
    fn test_optimization_pipeline() {
        let context = create_test_context();
        let mut module = create_multiply_function(&context).unwrap();

        // Apply optimization pipeline
        optimize_module(&context, &mut module).unwrap();

        assert!(verify_module(&module));
        let ir = module_to_string(&module);
        assert!(ir.contains("func.func @multiply"));
    }

    #[test]
    fn test_pass_pipeline_builder() {
        let context = create_test_context();
        let mut module = create_constant(&context, 100).unwrap();

        let pipeline = PassPipeline::new(&context)
            .canonicalize()
            .eliminate_common_subexpressions()
            .propagate_constants();

        pipeline.run(&mut module).unwrap();
        assert!(verify_module(&module));
    }
}
