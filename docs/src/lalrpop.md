# LALRPOP

LALRPOP is an LR(1) parser generator for Rust that produces efficient, table-driven parsers from declarative grammars. Unlike parser combinators, LALRPOP generates parsers at compile time from grammar files, providing excellent parsing performance and compile-time validation of grammar correctness. The generated parsers handle left recursion naturally and provide precise error messages with conflict detection during grammar compilation.

For compiler development, LALRPOP offers the traditional parser generator experience familiar to users of yacc, bison, or ANTLR, but with Rust's type safety and zero-cost abstractions. The generated code integrates seamlessly with Rust's ownership system, producing typed ASTs without runtime overhead. LALRPOP excels at parsing programming languages where performance matters and grammar complexity requires the power of LR parsing.

## Basic Calculator

A simple arithmetic expression parser demonstrates LALRPOP's syntax:

```rust
use std::str::FromStr;

grammar;

pub Expr: i32 = {
    <l:Expr> "+" <r:Factor> => l + r,
    <l:Expr> "-" <r:Factor> => l - r,
    Factor,
};

Factor: i32 = {
    <l:Factor> "*" <r:Term> => l * r,
    <l:Factor> "/" <r:Term> => l / r,
    Term,
};

Term: i32 = {
    Number,
    "(" <Expr> ")",
};

Number: i32 = {
    r"[0-9]+" => i32::from_str(<>).unwrap(),
};
```

This grammar correctly handles operator precedence through rule stratification. Addition and subtraction are parsed at a lower precedence level than multiplication and division. The parser is left-associative, parsing `10 - 2 - 3` as `(10 - 2) - 3 = 5`.

## AST Construction

Building typed ASTs requires defining the types and constructing them in grammar actions:

```rust
#![enum!("lalrpop/src/ast.rs", Expr)]
```

The grammar constructs this AST:

```rust
use crate::ast::{Expr};

grammar;

pub Expr: Expr = {
    <l:Expr> "+" <r:Term> => Expr::Add(Box::new(l), Box::new(r)),
    <l:Expr> "-" <r:Term> => Expr::Subtract(Box::new(l), Box::new(r)),
    Term,
};

Term: Expr = {
    <l:Term> "*" <r:Factor> => Expr::Multiply(Box::new(l), Box::new(r)),
    <l:Term> "/" <r:Factor> => Expr::Divide(Box::new(l), Box::new(r)),
    Factor,
};

Factor: Expr = {
    Primary,
    "-" <e:Factor> => Expr::Negate(Box::new(e)),
};

Primary: Expr = {
    Number => Expr::Number(<>),
    Identifier => Expr::Variable(<>),
    "(" <Expr> ")",
};

Number: f64 = {
    r"[0-9]+(\.[0-9]+)?" => f64::from_str(<>).unwrap(),
};

Identifier: String = {
    r"[a-zA-Z][a-zA-Z0-9_]*" => <>.to_string(),
};
```

Each production rule returns a value of the specified type. The angle bracket syntax `<name:Rule>` binds matched values to variables used in the action code.

## Statement Grammar

A more complete language with statements demonstrates complex grammar patterns:

```rust
#![struct!("lalrpop/src/ast.rs", Program)]
```

```rust
#![enum!("lalrpop/src/ast.rs", Statement)]
```

The grammar for this language:

```rust
Statement: Statement = {
    <e:Expr> ";" => Statement::Expression(e),
    "let" <name:Identifier> "=" <e:Expr> ";" => Statement::Assignment(name, e),
    "print" <e:Expr> ";" => Statement::Print(e),
    "if" <cond:Expr> "{" <then:Statement*> "}" <els:("else" "{" <Statement*> "}")?> => {
        Statement::If(cond, then, els)
    },
    "while" <cond:Expr> "{" <body:Statement*> "}" => Statement::While(cond, body),
};
```

The `*` operator creates lists of zero or more items. The `?` operator makes productions optional. Parentheses group sub-patterns for clarity.

## Using External Lexers

LALRPOP can use external lexers like logos for improved performance and features:

```rust
#![enum!("lalrpop/src/token.rs", Token)]
```

The grammar declares the external token type:

```rust
extern {
    type Location = usize;
    type Error = String;

    enum Token {
        "+" => Token::Plus,
        "-" => Token::Minus,
        "*" => Token::Star,
        "/" => Token::Slash,
        "=" => Token::Equals,
        "(" => Token::LeftParen,
        ")" => Token::RightParen,
        "number" => Token::Number(<f64>),
        "identifier" => Token::Identifier(<String>),
    }
}
```

Terminal symbols in the grammar now refer to token variants. The angle brackets extract associated data from token variants.

## Integration with Logos

Connecting logos lexer to LALRPOP parser:

```rust
#![function!("lalrpop/src/lib.rs", parse_with_logos)]
```

The lexer produces tokens with location information that LALRPOP uses for error reporting. This separation of concerns allows optimizing lexer and parser independently.

## Helper Rules

Common patterns can be abstracted into parameterized rules:

```rust
Comma<T>: Vec<T> = {
    <mut v:(<T> ",")*> <e:T?> => match e {
        None => v,
        Some(e) => {
            v.push(e);
            v
        }
    }
};
```

This generic rule parses comma-separated lists of any type. Use it like `<args:Comma<Expr>>` to parse function arguments.

## Build Configuration

LALRPOP requires a build script to generate parsers:

```rust
#![source_file!("lalrpop/build.rs")]
```

The build script processes all `.lalrpop` files in the source tree, generating corresponding Rust modules.

## Using Generated Parsers

Import generated parsers with the lalrpop_mod macro and use them to parse input:

```rust
#![function!("lalrpop/src/lib.rs", parse_calculator)]
```

Each public rule in the grammar generates a corresponding parser struct with a `parse` method.

## Error Handling

LALRPOP provides detailed error information with location tracking and expected tokens:

```rust
#![function!("lalrpop/src/lib.rs", parse_with_detailed_errors)]
```

The error types include location information and expected tokens, enabling high-quality error messages. The parser tracks byte positions which can be converted to line and column numbers for user-friendly error reporting.

## Precedence and Associativity

Operator precedence is controlled by grammar structure:

```rust
// Lower precedence
Expr: Expr = {
    <l:Expr> "||" <r:AndExpr> => Expr::Or(Box::new(l), Box::new(r)),
    AndExpr,
};

AndExpr: Expr = {
    <l:AndExpr> "&&" <r:CmpExpr> => Expr::And(Box::new(l), Box::new(r)),
    CmpExpr,
};

// Higher precedence
CmpExpr: Expr = {
    <l:CmpExpr> "==" <r:AddExpr> => Expr::Equal(Box::new(l), Box::new(r)),
    <l:CmpExpr> "!=" <r:AddExpr> => Expr::NotEqual(Box::new(l), Box::new(r)),
    AddExpr,
};
```

Rules lower in the grammar hierarchy have higher precedence. Left recursion creates left associativity; right recursion creates right associativity.

## Left Recursion

Unlike recursive descent parsers and PEG parsers, LALRPOP handles left recursion naturally and efficiently. This is a fundamental advantage of LR parsing that enables intuitive grammar definitions for left-associative operators and list constructions.

Consider the difference between left and right associative parsing for subtraction:

```rust
// LEFT RECURSIVE - parses "10 - 5 - 2" as (10 - 5) - 2 = 3
pub LeftAssociative: Expr = {
    <l:LeftAssociative> "-" <r:Term> => Expr::Subtract(Box::new(l), Box::new(r)),
    Term,
};

// RIGHT RECURSIVE - parses "10 - 5 - 2" as 10 - (5 - 2) = 7
pub RightAssociative: Expr = {
    <l:Term> "-" <r:RightAssociative> => Expr::Subtract(Box::new(l), Box::new(r)),
    Term,
};
```

The left recursive version correctly implements the standard mathematical interpretation where operations associate left to right. This natural expression of grammar rules is impossible in top-down parsers without transformation.

Left recursion excels at parsing lists that build incrementally:

```rust
// Builds list as items are encountered
pub CommaSeparatedLeft: Vec<i32> = {
    <mut list:CommaSeparatedLeft> "," <item:Number> => {
        list.push(item);
        list
    },
    <n:Number> => vec![n],
};
```

Field access and method chaining naturally use left recursion:

```rust
// Parses "obj.field1.field2" correctly
pub FieldAccess: String = {
    <obj:FieldAccess> "." <field:Identifier> => format!("{}.{}", obj, field),
    Identifier,
};

// Parses "obj.method1().method2()" correctly
pub MethodChain: String = {
    <obj:MethodChain> "." <method:Identifier> "(" ")" => format!("{}.{}()", obj, method),
    Identifier,
};
```

These patterns appear frequently in programming languages where operations chain from left to right. The ability to express them directly as left recursive rules simplifies grammar development and improves parser performance.

Postfix operators also benefit from left recursion:

```rust
// Array indexing, function calls, and postfix increment
PostfixExpr: Expr = {
    <e:PostfixExpr> "[" <index:Expr> "]" => Expr::Index(Box::new(e), Box::new(index)),
    <func:PostfixExpr> "(" <args:Arguments> ")" => Expr::Call(Box::new(func), args),
    <e:PostfixExpr> "++" => Expr::PostIncrement(Box::new(e)),
    PrimaryExpr,
};
```

Testing associativity demonstrates the difference:

```rust
#![function!("lalrpop/src/lib.rs", demonstrate_associativity)]
```

The function parses the same input with both left and right associative grammars, revealing how the parse tree structure differs. For the expression "10 - 5 - 2", left association produces 3 while right association produces 7.

Complex expressions with multiple precedence levels all use left recursion:

```rust
BinaryOp: Expr = {
    <l:BinaryOp> "||" <r:AndExpr> => Expr::Or(Box::new(l), Box::new(r)),
    AndExpr,
};

AndExpr: Expr = {
    <l:AndExpr> "&&" <r:EqExpr> => Expr::And(Box::new(l), Box::new(r)),
    EqExpr,
};

AddExpr: Expr = {
    <l:AddExpr> "+" <r:MulExpr> => Expr::Add(Box::new(l), Box::new(r)),
    <l:AddExpr> "-" <r:MulExpr> => Expr::Subtract(Box::new(l), Box::new(r)),
    MulExpr,
};
```

Each level of the precedence hierarchy uses left recursion to ensure operators associate correctly. This pattern scales to arbitrarily complex expression grammars while maintaining readability and performance.

The LR parsing algorithm builds the parse tree bottom-up, naturally handling left recursion without stack overflow issues that plague recursive descent parsers. This fundamental difference makes LALRPOP ideal for parsing programming languages with complex expression syntax.

## Building an Interpreter

LALRPOP-generated parsers integrate well with interpreters:

```rust
#![struct!("lalrpop/src/lib.rs", Interpreter)]
```

```rust
#![impl!("lalrpop/src/lib.rs", Interpreter)]
```

The interpreter walks the AST, maintaining variable bindings and executing statements. This separation of parsing and execution allows optimization and analysis passes between parsing and execution.

## Conflict Resolution

LALRPOP detects grammar conflicts at compile time:

```
error: ambiguity detected

  The following symbols can be reduced in two ways:
    Expr "+" Expr

  They could be reduced like so:
    Expr = Expr "+" Expr

  Or they could be reduced like so:
    Expr = Expr, "+" Expr
```

Resolve conflicts by restructuring the grammar or using precedence annotations. LALRPOP's error messages pinpoint the exact productions causing conflicts.

## Performance Optimization

LALRPOP generates table-driven parsers with excellent performance characteristics. The parsing algorithm is O(n) for valid input with no backtracking. Tables are computed at compile time, so runtime overhead is minimal.

For maximum performance, use external lexers like logos that produce tokens in a single pass. The combination of logos lexing and LALRPOP parsing can process millions of lines per second.

## Best Practices

Structure grammars for clarity and maintainability. Group related productions together and use comments to explain complex patterns. Keep action code simple, delegating complex logic to separate functions.

Use typed ASTs to catch errors at compile time. The type system ensures grammar productions and AST construction remain synchronized. Changes to the AST that break the grammar are caught during compilation.

Test grammars thoroughly with both valid and invalid input. LALRPOP's error reporting helps debug grammar issues, but comprehensive tests ensure the parser accepts the intended language.

Profile parser performance on realistic input. While LALRPOP generates efficient parsers, grammar structure affects performance. Minimize ambiguity and left-factorize common prefixes when performance matters.

The combination of LALRPOP's powerful grammar language, excellent error messages, and efficient generated code makes it ideal for production compiler implementations. The traditional parser generator approach provides familiarity for developers experienced with other parsing tools while leveraging Rust's safety and performance advantages.