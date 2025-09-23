use std::hash::Hash;

use indexmap::map::Entry;
use indexmap::{IndexMap, IndexSet};

#[derive(Debug, Clone, PartialEq)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub scope_level: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SymbolKind {
    Variable { mutable: bool, ty: String },
    Function { params: Vec<String>, ret_ty: String },
    Type { definition: String },
}

pub struct SymbolTable {
    scopes: Vec<IndexMap<String, Symbol>>,
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            scopes: vec![IndexMap::new()],
        }
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(IndexMap::new());
    }

    pub fn pop_scope(&mut self) -> Option<IndexMap<String, Symbol>> {
        if self.scopes.len() > 1 {
            self.scopes.pop()
        } else {
            None
        }
    }

    pub fn insert(&mut self, name: String, symbol: Symbol) -> Option<Symbol> {
        // We maintain the invariant that there's always at least one scope
        debug_assert!(
            !self.scopes.is_empty(),
            "SymbolTable should always have at least one scope"
        );
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, symbol)
        } else {
            None
        }
    }

    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        for scope in self.scopes.iter().rev() {
            if let Some(symbol) = scope.get(name) {
                return Some(symbol);
            }
        }
        None
    }

    pub fn current_scope_symbols(&self) -> Vec<(&String, &Symbol)> {
        // We maintain the invariant that there's always at least one scope
        debug_assert!(
            !self.scopes.is_empty(),
            "SymbolTable should always have at least one scope"
        );
        self.scopes
            .last()
            .map(|scope| scope.iter().collect())
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone)]
pub struct StructField {
    pub name: String,
    pub ty: String,
    pub offset: usize,
}

pub fn create_struct_layout() -> IndexMap<String, StructField> {
    let mut fields = IndexMap::new();

    fields.insert(
        "id".to_string(),
        StructField {
            name: "id".to_string(),
            ty: "u64".to_string(),
            offset: 0,
        },
    );

    fields.insert(
        "name".to_string(),
        StructField {
            name: "name".to_string(),
            ty: "String".to_string(),
            offset: 8,
        },
    );

    fields.insert(
        "data".to_string(),
        StructField {
            name: "data".to_string(),
            ty: "Vec<u8>".to_string(),
            offset: 32,
        },
    );

    fields
}

pub struct ImportResolver {
    imports: IndexMap<String, IndexSet<String>>,
}

impl Default for ImportResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl ImportResolver {
    pub fn new() -> Self {
        Self {
            imports: IndexMap::new(),
        }
    }

    pub fn add_import(&mut self, module: String, items: Vec<String>) {
        match self.imports.entry(module) {
            Entry::Occupied(mut e) => {
                for item in items {
                    e.get_mut().insert(item);
                }
            }
            Entry::Vacant(e) => {
                let mut set = IndexSet::new();
                for item in items {
                    set.insert(item);
                }
                e.insert(set);
            }
        }
    }

    pub fn get_imports(&self) -> Vec<(String, Vec<String>)> {
        self.imports
            .iter()
            .map(|(module, items)| (module.clone(), items.iter().cloned().collect()))
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct TypeDefinition {
    pub name: String,
    pub kind: TypeKind,
}

#[derive(Debug, Clone)]
pub enum TypeKind {
    Primitive,
    Struct { fields: IndexMap<String, String> },
    Enum { variants: IndexSet<String> },
    Alias { target: String },
}

pub struct TypeRegistry {
    types: IndexMap<String, TypeDefinition>,
}

impl Default for TypeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            types: IndexMap::new(),
        };

        registry.types.insert(
            "i32".to_string(),
            TypeDefinition {
                name: "i32".to_string(),
                kind: TypeKind::Primitive,
            },
        );

        registry.types.insert(
            "bool".to_string(),
            TypeDefinition {
                name: "bool".to_string(),
                kind: TypeKind::Primitive,
            },
        );

        registry.types.insert(
            "String".to_string(),
            TypeDefinition {
                name: "String".to_string(),
                kind: TypeKind::Primitive,
            },
        );

        registry
    }

    pub fn register_type(&mut self, def: TypeDefinition) -> bool {
        match self.types.entry(def.name.clone()) {
            Entry::Vacant(e) => {
                e.insert(def);
                true
            }
            Entry::Occupied(_) => false,
        }
    }

    pub fn get_type(&self, name: &str) -> Option<&TypeDefinition> {
        self.types.get(name)
    }

    pub fn iter_types(&self) -> impl Iterator<Item = (&String, &TypeDefinition)> {
        self.types.iter()
    }
}

pub fn demonstrate_field_ordering() {
    let fields = create_struct_layout();

    println!("Struct fields in definition order:");
    for (i, (name, field)) in fields.iter().enumerate() {
        println!(
            "  {}: {} ({}) at offset {}",
            i, name, field.ty, field.offset
        );
    }

    println!();
    println!("Field access by name:");
    if let Some(field) = fields.get("name") {
        println!("  fields[\"name\"] = {:?}", field);
    }

    println!();
    println!("Field access by index:");
    if let Some((_name, field)) = fields.get_index(1) {
        println!("  fields[1] = {:?}", field);
    }
}

pub fn demonstrate_import_resolution() {
    let mut resolver = ImportResolver::new();

    resolver.add_import(
        "std::collections".to_string(),
        vec!["HashMap".to_string(), "Vec".to_string()],
    );

    resolver.add_import(
        "std::io".to_string(),
        vec!["Read".to_string(), "Write".to_string()],
    );

    resolver.add_import("std::collections".to_string(), vec!["HashSet".to_string()]);

    println!("Import resolution order:");
    for (module, items) in resolver.get_imports() {
        println!("  use {} {{ {} }};", module, items.join(", "));
    }
}

pub struct LocalScope<K: Hash + Eq, V> {
    bindings: IndexMap<K, V>,
}

impl<K: Hash + Eq + Clone, V> Default for LocalScope<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Hash + Eq + Clone, V> LocalScope<K, V> {
    pub fn new() -> Self {
        Self {
            bindings: IndexMap::new(),
        }
    }

    pub fn bind(&mut self, name: K, value: V) -> Option<V> {
        self.bindings.insert(name, value)
    }

    pub fn lookup(&self, name: &K) -> Option<&V> {
        self.bindings.get(name)
    }

    pub fn bindings_ordered(&self) -> Vec<(K, &V)> {
        self.bindings.iter().map(|(k, v)| (k.clone(), v)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_table_scoping() {
        let mut table = SymbolTable::new();

        table.insert(
            "x".to_string(),
            Symbol {
                name: "x".to_string(),
                kind: SymbolKind::Variable {
                    mutable: true,
                    ty: "i32".to_string(),
                },
                scope_level: 0,
            },
        );

        table.push_scope();

        table.insert(
            "x".to_string(),
            Symbol {
                name: "x".to_string(),
                kind: SymbolKind::Variable {
                    mutable: false,
                    ty: "bool".to_string(),
                },
                scope_level: 1,
            },
        );

        assert_eq!(table.lookup("x").unwrap().scope_level, 1);

        table.pop_scope();

        assert_eq!(table.lookup("x").unwrap().scope_level, 0);
    }

    #[test]
    fn test_struct_field_ordering() {
        let fields = create_struct_layout();

        let keys: Vec<_> = fields.keys().cloned().collect();
        assert_eq!(keys, vec!["id", "name", "data"]);

        assert_eq!(fields.get_index(0).unwrap().0, "id");
        assert_eq!(fields.get_index(1).unwrap().0, "name");
        assert_eq!(fields.get_index(2).unwrap().0, "data");
    }

    #[test]
    fn test_type_registry() {
        let mut registry = TypeRegistry::new();

        let struct_def = TypeDefinition {
            name: "Point".to_string(),
            kind: TypeKind::Struct {
                fields: IndexMap::from([
                    ("x".to_string(), "f64".to_string()),
                    ("y".to_string(), "f64".to_string()),
                ]),
            },
        };

        assert!(registry.register_type(struct_def.clone()));
        assert!(!registry.register_type(struct_def));

        assert!(registry.get_type("Point").is_some());
        assert!(registry.get_type("i32").is_some());
    }
}
