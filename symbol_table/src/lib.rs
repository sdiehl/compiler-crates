use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use symbol_table::{static_symbol, GlobalSymbol, Symbol, SymbolTable};

#[derive(Debug, Clone)]
pub struct Identifier {
    pub symbol: GlobalSymbol,
    pub span: Span,
}

#[derive(Debug, Clone, Copy)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Identifier {
    pub fn new(name: &str, span: Span) -> Self {
        Self {
            symbol: GlobalSymbol::from(name),
            span,
        }
    }

    pub fn as_str(&self) -> &str {
        self.symbol.as_str()
    }
}

pub fn demonstrate_static_symbols() {
    let if_sym = static_symbol!("if");
    let else_sym = static_symbol!("else");
    let while_sym = static_symbol!("while");
    let for_sym = static_symbol!("for");
    let return_sym = static_symbol!("return");

    println!("Static symbols created:");
    println!("  if: {:?}", if_sym);
    println!("  else: {:?}", else_sym);
    println!("  while: {:?}", while_sym);
    println!("  for: {:?}", for_sym);
    println!("  return: {:?}", return_sym);

    let if_sym2 = static_symbol!("if");
    println!("\nSymbol equality: if == if2: {}", if_sym == if_sym2);
    println!(
        "Pointer equality: {:p} == {:p}",
        if_sym.as_str(),
        if_sym2.as_str()
    );
}

pub fn demonstrate_global_symbols() {
    let sym1 = GlobalSymbol::from("variable_name");
    let sym2 = GlobalSymbol::from("variable_name");
    let sym3 = GlobalSymbol::from("another_name");

    println!("Symbol interning:");
    println!("  sym1 == sym2: {}", sym1 == sym2);
    println!("  sym1 == sym3: {}", sym1 == sym3);
    println!(
        "  Pointers equal: {}",
        std::ptr::eq(sym1.as_str(), sym2.as_str())
    );
}

pub struct CompilerContext {
    pub symbols: SymbolTable,
    pub keywords: HashMap<Symbol, TokenKind>,
    pub string_literals: Vec<Symbol>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    If,
    Else,
    While,
    For,
    Return,
    Function,
    Let,
    Const,
}

impl Default for CompilerContext {
    fn default() -> Self {
        Self::new()
    }
}

impl CompilerContext {
    pub fn new() -> Self {
        let symbols = SymbolTable::new();
        let mut keywords = HashMap::new();

        keywords.insert(symbols.intern("if"), TokenKind::If);
        keywords.insert(symbols.intern("else"), TokenKind::Else);
        keywords.insert(symbols.intern("while"), TokenKind::While);
        keywords.insert(symbols.intern("for"), TokenKind::For);
        keywords.insert(symbols.intern("return"), TokenKind::Return);
        keywords.insert(symbols.intern("function"), TokenKind::Function);
        keywords.insert(symbols.intern("let"), TokenKind::Let);
        keywords.insert(symbols.intern("const"), TokenKind::Const);

        Self {
            symbols,
            keywords,
            string_literals: Vec::new(),
        }
    }

    pub fn intern_string(&mut self, s: &str) -> Symbol {
        self.symbols.intern(s)
    }

    pub fn is_keyword(&self, sym: Symbol) -> Option<&TokenKind> {
        self.keywords.get(&sym)
    }

    pub fn add_string_literal(&mut self, s: &str) -> usize {
        let sym = self.symbols.intern(s);
        self.string_literals.push(sym);
        self.string_literals.len() - 1
    }
}

pub fn demonstrate_compiler_context() {
    let mut ctx = CompilerContext::new();

    let ident = ctx.intern_string("my_variable");
    let keyword = ctx.intern_string("if");

    println!("Identifier 'my_variable' interned as Symbol");
    println!("Is 'my_variable' a keyword: {:?}", ctx.is_keyword(ident));
    println!("Is 'if' a keyword: {:?}", ctx.is_keyword(keyword));

    let lit_idx = ctx.add_string_literal("Hello, world!");
    println!("String literal index: {}", lit_idx);
}

#[derive(Debug)]
pub struct ModuleSymbolTable {
    pub name: GlobalSymbol,
    pub exported: HashMap<GlobalSymbol, SymbolInfo>,
    pub internal: HashMap<GlobalSymbol, SymbolInfo>,
}

#[derive(Debug, Clone)]
pub struct SymbolInfo {
    pub kind: SymbolKind,
    pub defined_at: Option<Location>,
    pub type_info: Option<String>,
}

#[derive(Debug, Clone)]
pub enum SymbolKind {
    Function,
    Variable,
    Type,
    Module,
}

#[derive(Debug, Clone)]
pub struct Location {
    pub file: GlobalSymbol,
    pub line: u32,
    pub column: u32,
}

impl ModuleSymbolTable {
    pub fn new(name: &str) -> Self {
        Self {
            name: GlobalSymbol::from(name),
            exported: HashMap::new(),
            internal: HashMap::new(),
        }
    }

    pub fn define_exported(&mut self, name: &str, info: SymbolInfo) {
        self.exported.insert(GlobalSymbol::from(name), info);
    }

    pub fn define_internal(&mut self, name: &str, info: SymbolInfo) {
        self.internal.insert(GlobalSymbol::from(name), info);
    }

    pub fn lookup(&self, name: &GlobalSymbol) -> Option<&SymbolInfo> {
        self.exported.get(name).or_else(|| self.internal.get(name))
    }
}

pub type ConcurrentSymbolCache = Arc<RwLock<HashMap<GlobalSymbol, String>>>;

pub fn create_concurrent_cache() -> ConcurrentSymbolCache {
    Arc::new(RwLock::new(HashMap::new()))
}

pub fn demonstrate_concurrent_access() {
    let cache = create_concurrent_cache();
    let symbols: Vec<_> = (0..10)
        .map(|i| GlobalSymbol::from(format!("symbol_{}", i)))
        .collect();

    {
        let mut cache_write = cache.write().unwrap();
        for (i, sym) in symbols.iter().enumerate() {
            cache_write.insert(*sym, format!("Value {}", i));
        }
    }

    {
        let cache_read = cache.read().unwrap();
        println!("Concurrent cache contents:");
        for sym in &symbols[..5] {
            if let Some(value) = cache_read.get(sym) {
                println!("  {} => {}", sym.as_str(), value);
            }
        }
    }
}

pub fn benchmark_symbol_creation() {
    use std::time::Instant;

    let iterations = 10000;
    let unique_symbols = 1000;

    let start = Instant::now();
    let table = SymbolTable::new();
    for i in 0..iterations {
        let name = format!("symbol_{}", i % unique_symbols);
        table.intern(&name);
    }
    let local_time = start.elapsed();

    let start = Instant::now();
    for i in 0..iterations {
        let name = format!("symbol_{}", i % unique_symbols);
        let _ = GlobalSymbol::from(name);
    }
    let global_time = start.elapsed();

    println!(
        "Symbol creation benchmark ({} iterations, {} unique):",
        iterations, unique_symbols
    );
    println!("  Local SymbolTable: {:?}", local_time);
    println!("  GlobalSymbol: {:?}", global_time);
    println!(
        "  Ratio: {:.2}x",
        local_time.as_nanos() as f64 / global_time.as_nanos() as f64
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_static_symbol_equality() {
        let sym1 = static_symbol!("test");
        let sym2 = static_symbol!("test");
        let sym3 = GlobalSymbol::from("test");

        assert_eq!(sym1, sym2);
        assert_eq!(sym1, sym3);
        assert!(std::ptr::eq(sym1.as_str(), sym2.as_str()));
    }

    #[test]
    fn test_identifier() {
        let ident = Identifier::new("foo", Span { start: 0, end: 3 });
        assert_eq!(ident.as_str(), "foo");
        assert_eq!(ident.symbol, GlobalSymbol::from("foo"));
    }

    #[test]
    fn test_compiler_context() {
        let mut ctx = CompilerContext::new();
        let sym = ctx.intern_string("if");

        assert!(ctx.is_keyword(sym).is_some());
        assert_eq!(*ctx.is_keyword(sym).unwrap(), TokenKind::If);

        let var_sym = ctx.intern_string("my_var");
        assert!(ctx.is_keyword(var_sym).is_none());
    }

    #[test]
    fn test_module_symbol_table() {
        let mut module = ModuleSymbolTable::new("my_module");

        module.define_exported(
            "public_func",
            SymbolInfo {
                kind: SymbolKind::Function,
                defined_at: None,
                type_info: Some("fn() -> i32".to_string()),
            },
        );

        module.define_internal(
            "private_var",
            SymbolInfo {
                kind: SymbolKind::Variable,
                defined_at: None,
                type_info: Some("String".to_string()),
            },
        );

        let pub_sym = GlobalSymbol::from("public_func");
        let priv_sym = GlobalSymbol::from("private_var");

        assert!(module.lookup(&pub_sym).is_some());
        assert!(module.lookup(&priv_sym).is_some());
        assert!(module.exported.contains_key(&pub_sym));
        assert!(!module.exported.contains_key(&priv_sym));
    }
}
