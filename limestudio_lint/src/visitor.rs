use syn::visit::{self, Visit};
use syn::{Expr, Stmt, ItemFn, Macro};

pub struct LimeVisitor<'a> {
    pub violations: Vec<Violation>,
    pub file_path: &'a str,
}

pub struct Violation {
    pub message: String,
    pub rule: String,
    pub line: usize,
}

impl<'a> Visit<'a> for LimeVisitor<'a> {
    fn visit_macro(&mut self, i: &'a Macro) {
        if i.path.is_ident("plugin") {
            // Further inspection of plugin! macro body
            // This would require token parsing
        }
        visit::visit_macro(self, i);
    }

    fn visit_stmt(&mut self, i: &'a Stmt) {
        // Detect local state mutations like self.local_gain = ...
        // (Simplified for now)
        visit::visit_stmt(self, i);
    }

    fn visit_item_fn(&mut self, i: &'a ItemFn) {
        // Inspect dsp functions
        if i.sig.ident == "process" || i.sig.ident == "dsp" {
            // Check for println!, panic!, Mutex::lock, etc.
        }
        visit::visit_item_fn(self, i);
    }
}

pub fn check_realtime_safety(tokens: &str) -> Vec<String> {
    let mut bad_calls = Vec::new();
    if tokens.contains("println!") { bad_calls.push("println!".into()); }
    if tokens.contains("Mutex::lock") { bad_calls.push("Mutex::lock".into()); }
    if tokens.contains("Vec::push") { bad_calls.push("Vec::push".into()); }
    bad_calls
}
