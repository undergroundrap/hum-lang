//! WO28 #16: lexical scope stack for type facts, shared by `type_check.rs`
//! (return checking) and `full_type_check.rs` (per-statement facts).
//!
//! Block scoping must match the resolver: `if`/`while`/`for-each`/`for-index`/
//! `loop` headers push a fresh scope, `block_close` pops it. Bindings inside a
//! block do not leak out, and shadowed outer facts are restored when the block
//! closes. The for-each binder goes into the pushed scope.

use std::collections::BTreeMap;

/// A lexical scope stack for type facts. The base scope (task parameters)
/// is never popped.
pub(crate) struct TypeScopeStack<T> {
    scopes: Vec<BTreeMap<String, T>>,
}

impl<T: Clone> TypeScopeStack<T> {
    /// Create a stack with the given base scope (e.g. task parameters).
    pub(crate) fn new(base: BTreeMap<String, T>) -> Self {
        Self { scopes: vec![base] }
    }

    /// Push a fresh scope for a block opener. Call before processing the
    /// header's own bindings so they land in the new scope.
    pub(crate) fn push_block(&mut self) {
        self.scopes.push(BTreeMap::new());
    }

    /// Pop a block scope, restoring shadowed outer facts. Never pops the
    /// base scope.
    pub(crate) fn pop_block(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    /// Handle block open/close for a statement kind: push on
    /// `if`/`while`/`for-each`/`for-index`/`loop` headers, pop on
    /// `block_close`.
    pub(crate) fn handle_block_boundary(&mut self, kind: &str) {
        match kind {
            "if_header" | "while_header" | "for_each_header" | "for_index_header"
            | "loop_header" => {
                self.push_block();
            }
            "block_close" => {
                self.pop_block();
            }
            _ => {}
        }
    }

    /// Insert a fact into the current (innermost) scope.
    pub(crate) fn insert(&mut self, name: &str, fact: T) {
        if let Some(top) = self.scopes.last_mut() {
            top.insert(snake_identifier(name), fact);
        }
    }

    /// Look up a fact from innermost to outermost scope, matching resolver
    /// block scoping.
    pub(crate) fn lookup(&self, name: &str) -> Option<T> {
        let key = snake_identifier(name);
        for scope in self.scopes.iter().rev() {
            if let Some(fact) = scope.get(&key) {
                return Some(fact.clone());
            }
        }
        None
    }
}

fn snake_identifier(text: &str) -> String {
    let mut out = String::new();
    let mut previous_was_separator = false;
    for ch in text.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            previous_was_separator = false;
        } else if !previous_was_separator && !out.is_empty() {
            out.push('_');
            previous_was_separator = true;
        }
    }
    out.trim_matches('_').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_scope_restores_shadowed_fact() {
        let mut base = BTreeMap::new();
        base.insert("x".to_string(), "outer".to_string());
        let mut stack = TypeScopeStack::new(base);

        stack.handle_block_boundary("if_header");
        stack.insert("x", "inner".to_string());
        assert_eq!(stack.lookup("x"), Some("inner".to_string()));

        stack.handle_block_boundary("block_close");
        assert_eq!(stack.lookup("x"), Some("outer".to_string()));
    }

    #[test]
    fn block_scope_does_not_leak_inner_binding() {
        let mut stack: TypeScopeStack<String> = TypeScopeStack::new(BTreeMap::new());

        stack.handle_block_boundary("for_each_header");
        stack.insert("y", "inner".to_string());
        assert_eq!(stack.lookup("y"), Some("inner".to_string()));

        stack.handle_block_boundary("block_close");
        assert_eq!(stack.lookup("y"), None);
    }

    #[test]
    fn block_close_never_pops_base_scope() {
        let mut base = BTreeMap::new();
        base.insert("x".to_string(), "outer".to_string());
        let mut stack = TypeScopeStack::new(base);

        stack.handle_block_boundary("block_close");
        stack.handle_block_boundary("block_close");
        assert_eq!(stack.lookup("x"), Some("outer".to_string()));
    }

    #[test]
    fn nested_blocks_restore_in_order() {
        let mut stack: TypeScopeStack<String> = TypeScopeStack::new(BTreeMap::new());

        stack.handle_block_boundary("if_header");
        stack.insert("x", "level1".to_string());
        stack.handle_block_boundary("while_header");
        stack.insert("x", "level2".to_string());
        assert_eq!(stack.lookup("x"), Some("level2".to_string()));

        stack.handle_block_boundary("block_close");
        assert_eq!(stack.lookup("x"), Some("level1".to_string()));

        stack.handle_block_boundary("block_close");
        assert_eq!(stack.lookup("x"), None);
    }
}
