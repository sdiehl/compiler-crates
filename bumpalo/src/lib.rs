use std::fmt::Debug;

use bumpalo::Bump;
use bumpalo::boxed::Box as BumpBox;
use bumpalo::collections::Vec as BumpVec;

/// Demonstrates basic bump allocation with simple types
pub fn basic_allocation() -> Vec<i32> {
    let bump = Bump::new();

    // Allocate individual values
    let x = bump.alloc(10);
    let y = bump.alloc(20);
    let z = bump.alloc(30);

    // Values are valid for the lifetime of the bump allocator
    vec![*x, *y, *z]
}

/// Shows how to allocate strings in the bump allocator
pub fn allocate_strings(bump: &Bump) -> &str {
    // Allocate a string slice
    let hello = bump.alloc_str("Hello, ");
    let world = bump.alloc_str("World!");

    // Concatenate using bump allocation

    (bump.alloc_str(&format!("{}{}", hello, world))) as _
}

/// Allocates slices efficiently in the bump allocator
pub fn allocate_slices(bump: &Bump) -> &[i32] {
    // Allocate a slice from a vector
    let data = vec![1, 2, 3, 4, 5];
    bump.alloc_slice_copy(&data)
}

/// Demonstrates using bump-allocated collections
pub fn bump_collections() -> Vec<i32> {
    let bump = Bump::new();

    // Create a bump-allocated vector
    let mut vec = BumpVec::new_in(&bump);
    vec.push(1);
    vec.push(2);
    vec.push(3);

    // Convert to standard Vec for return
    vec.iter().copied().collect()
}

/// Shows arena-style allocation for AST nodes
#[derive(Debug, Clone)]
pub enum Expr<'a> {
    Number(i64),
    Add(&'a Expr<'a>, &'a Expr<'a>),
    Multiply(&'a Expr<'a>, &'a Expr<'a>),
}

pub fn build_ast<'a>(bump: &'a Bump) -> &'a Expr<'a> {
    // Build expression: (2 + 3) * 4
    let two = bump.alloc(Expr::Number(2));
    let three = bump.alloc(Expr::Number(3));
    let four = bump.alloc(Expr::Number(4));

    let add = bump.alloc(Expr::Add(two, three));
    bump.alloc(Expr::Multiply(add, four))
}

/// Evaluates an AST expression
pub fn eval_expr(expr: &Expr) -> i64 {
    match expr {
        Expr::Number(n) => *n,
        Expr::Add(a, b) => eval_expr(a) + eval_expr(b),
        Expr::Multiply(a, b) => eval_expr(a) * eval_expr(b),
    }
}

/// Demonstrates using bump allocation for a simple compiler IR
pub struct Function<'a> {
    pub name: &'a str,
    pub params: BumpVec<'a, &'a str>,
    pub body: BumpVec<'a, Statement<'a>>,
}

pub enum Statement<'a> {
    Let(&'a str, &'a Expr<'a>),
    Return(&'a Expr<'a>),
}

pub fn build_function<'a>(bump: &'a Bump) -> Function<'a> {
    let mut params = BumpVec::new_in(bump);
    let x = bump.alloc_str("x");
    let y = bump.alloc_str("y");
    params.push(&*x);
    params.push(&*y);

    let mut body = BumpVec::new_in(bump);

    // let sum = x + y
    let x = bump.alloc(Expr::Number(10));
    let y = bump.alloc(Expr::Number(20));
    let sum_expr = bump.alloc(Expr::Add(x, y));
    body.push(Statement::Let("sum", sum_expr));

    // return sum * 2
    let two = bump.alloc(Expr::Number(2));
    let result = bump.alloc(Expr::Multiply(sum_expr, two));
    body.push(Statement::Return(result));

    Function {
        name: bump.alloc_str("calculate"),
        params,
        body,
    }
}

/// Shows how to reset and reuse a bump allocator
pub fn reset_and_reuse() -> (Vec<i32>, Vec<i32>) {
    let mut bump = Bump::new();

    // First allocation cycle
    let first = {
        let vec = BumpVec::new_in(&bump);
        let mut vec = vec;
        vec.extend([1, 2, 3].iter().copied());
        vec.iter().copied().collect::<Vec<_>>()
    };

    // Reset the allocator to reclaim all memory
    bump.reset();

    // Second allocation cycle reuses the same memory
    let second = {
        let vec = BumpVec::new_in(&bump);
        let mut vec = vec;
        vec.extend([4, 5, 6].iter().copied());
        vec.iter().copied().collect::<Vec<_>>()
    };

    (first, second)
}

/// Demonstrates scoped allocation for temporary computations
pub fn scoped_allocation() -> i32 {
    let bump = Bump::new();

    // Create a temporary allocation scope

    // Memory is automatically freed when bump goes out of scope
    {
        // Allocate temporary working data
        let mut temps = BumpVec::new_in(&bump);
        for i in 0..100 {
            temps.push(i);
        }

        // Process data
        temps.iter().sum::<i32>()
    }
}

/// Shows using bump allocation with closures
pub fn with_allocator<F, R>(f: F) -> R
where
    F: FnOnce(&Bump) -> R, {
    let bump = Bump::new();
    f(&bump)
}

pub fn closure_example() -> i32 {
    with_allocator(|bump| {
        let numbers = bump.alloc_slice_copy(&[1, 2, 3, 4, 5]);
        numbers.iter().sum()
    })
}

/// Custom type that uses bump allocation internally
pub struct SymbolTable<'a> {
    bump: &'a Bump,
    symbols: BumpVec<'a, &'a str>,
}

impl<'a> SymbolTable<'a> {
    pub fn new(bump: &'a Bump) -> Self {
        Self {
            bump,
            symbols: BumpVec::new_in(bump),
        }
    }

    pub fn intern(&mut self, s: &str) -> usize {
        // Check if symbol already exists
        for (i, &sym) in self.symbols.iter().enumerate() {
            if sym == s {
                return i;
            }
        }

        // Allocate new symbol
        let symbol = self.bump.alloc_str(s);
        let id = self.symbols.len();
        self.symbols.push(symbol);
        id
    }

    pub fn get(&self, id: usize) -> Option<&'a str> {
        self.symbols.get(id).copied()
    }
}

/// Demonstrates using bump allocation for graph structures
pub struct Node<'a> {
    pub value: i32,
    pub children: BumpVec<'a, &'a Node<'a>>,
}

pub fn build_tree<'a>(bump: &'a Bump) -> &'a Node<'a> {
    // Build a simple tree structure
    let leaf1 = bump.alloc(Node {
        value: 1,
        children: BumpVec::new_in(bump),
    });

    let leaf2 = bump.alloc(Node {
        value: 2,
        children: BumpVec::new_in(bump),
    });

    let mut branch_children = BumpVec::new_in(bump);
    branch_children.push(&*leaf1);
    branch_children.push(&*leaf2);

    bump.alloc(Node {
        value: 3,
        children: branch_children,
    })
}

/// Shows statistics about memory usage
pub fn allocation_stats() {
    let mut bump = Bump::new();

    // Allocate some data
    for i in 0..1000 {
        bump.alloc(i);
    }

    // Get allocation statistics
    let allocated = bump.allocated_bytes();
    println!("Allocated: {} bytes", allocated);

    // Reset and check again
    bump.reset();
    let after_reset = bump.allocated_bytes();
    println!("After reset: {} bytes", after_reset);
}

/// Demonstrates bump boxes for single-value allocation
pub fn bump_box_example() -> i32 {
    let bump = Bump::new();

    // Create a bump-allocated box
    let boxed: BumpBox<i32> = BumpBox::new_in(100, &bump);

    // Bump boxes can be dereferenced like regular boxes
    *boxed
}

/// Shows efficient string building with bump allocation
pub struct StringBuilder<'a> {
    bump: &'a Bump,
    parts: BumpVec<'a, &'a str>,
}

impl<'a> StringBuilder<'a> {
    pub fn new(bump: &'a Bump) -> Self {
        Self {
            bump,
            parts: BumpVec::new_in(bump),
        }
    }

    pub fn append(&mut self, s: &str) {
        let part = self.bump.alloc_str(s);
        self.parts.push(part);
    }

    pub fn build(&self) -> String {
        self.parts.iter().flat_map(|s| s.chars()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_allocation() {
        let result = basic_allocation();
        assert_eq!(result, vec![10, 20, 30]);
    }

    #[test]
    fn test_string_allocation() {
        let bump = Bump::new();
        let result = allocate_strings(&bump);
        assert_eq!(result, "Hello, World!");
    }

    #[test]
    fn test_ast_evaluation() {
        let bump = Bump::new();
        let ast = build_ast(&bump);
        assert_eq!(eval_expr(ast), 20); // (2 + 3) * 4 = 20
    }

    #[test]
    fn test_reset_reuse() {
        let (first, second) = reset_and_reuse();
        assert_eq!(first, vec![1, 2, 3]);
        assert_eq!(second, vec![4, 5, 6]);
    }

    #[test]
    fn test_symbol_table() {
        let bump = Bump::new();
        let mut table = SymbolTable::new(&bump);

        let id1 = table.intern("hello");
        let id2 = table.intern("world");
        let id3 = table.intern("hello"); // Should return same ID

        assert_eq!(id1, id3);
        assert_ne!(id1, id2);
        assert_eq!(table.get(id1), Some("hello"));
        assert_eq!(table.get(id2), Some("world"));
    }

    #[test]
    fn test_tree_building() {
        let bump = Bump::new();
        let tree = build_tree(&bump);

        assert_eq!(tree.value, 3);
        assert_eq!(tree.children.len(), 2);
        assert_eq!(tree.children[0].value, 1);
        assert_eq!(tree.children[1].value, 2);
    }

    #[test]
    fn test_string_builder() {
        let bump = Bump::new();
        let mut builder = StringBuilder::new(&bump);

        builder.append("Hello");
        builder.append(", ");
        builder.append("World!");

        assert_eq!(builder.build(), "Hello, World!");
    }
}
