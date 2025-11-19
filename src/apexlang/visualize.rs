use crate::apexlang::ast::{
    BinaryOp, Expr, Function, Import, ImportKind, Program, Statement, UnaryOp,
};
use crate::apexlang::error::ApexError;
use crate::apexlang::parser::Parser;

pub fn visualize_source(source: &str) -> Result<String, ApexError> {
    let program = Parser::parse(source)?;
    Ok(program_to_dot(&program))
}

pub fn program_to_dot(program: &Program) -> String {
    let mut builder = DotBuilder::default();
    let root = builder.add_node("Program");

    for (index, import) in program.imports.iter().enumerate() {
        let import_label = format!("Import #{}", index + 1);
        let import_id = builder.add_node(import_label);
        builder.add_edge(root, import_id);
        let detail_id = builder.add_node(describe_import(import));
        builder.add_edge(import_id, detail_id);
    }

    for function in &program.functions {
        let fn_id = add_function(&mut builder, function);
        builder.add_edge(root, fn_id);
    }

    builder.finish()
}

fn add_function(builder: &mut DotBuilder, function: &Function) -> usize {
    let params = if function.params.is_empty() {
        "".to_string()
    } else {
        function.params.join(", ")
    };
    let label = format!("fn {}({})", function.name, params);
    let fn_id = builder.add_node(label);
    for statement in &function.body {
        let stmt_id = add_statement(builder, statement);
        builder.add_edge(fn_id, stmt_id);
    }
    fn_id
}

fn add_statement(builder: &mut DotBuilder, statement: &Statement) -> usize {
    match statement {
        Statement::Let {
            name,
            mutable,
            value,
        } => {
            let label = if *mutable {
                format!("var {}", name)
            } else {
                format!("let {}", name)
            };
            let stmt_id = builder.add_node(label);
            let value_id = add_expr(builder, value);
            builder.add_edge(stmt_id, value_id);
            stmt_id
        }
        Statement::Assignment { name, value } => {
            let stmt_id = builder.add_node(format!("assign {}", name));
            let expr_id = add_expr(builder, value);
            builder.add_edge(stmt_id, expr_id);
            stmt_id
        }
        Statement::Expr(expr) => {
            let stmt_id = builder.add_node("expr");
            let expr_id = add_expr(builder, expr);
            builder.add_edge(stmt_id, expr_id);
            stmt_id
        }
        Statement::Return(expr) => {
            let stmt_id = builder.add_node("return");
            let expr_id = add_expr(builder, expr);
            builder.add_edge(stmt_id, expr_id);
            stmt_id
        }
    }
}

fn add_expr(builder: &mut DotBuilder, expr: &Expr) -> usize {
    match expr {
        Expr::Literal(value) => builder.add_node(format!("Literal: {}", value)),
        Expr::Path(path) => builder.add_node(format!("Path: {}", path.segments.join("::"))),
        Expr::Unary(op, inner) => {
            let node_id = builder.add_node(format!("Unary: {}", unary_symbol(*op)));
            let child = add_expr(builder, inner);
            builder.add_edge(node_id, child);
            node_id
        }
        Expr::Binary(lhs, op, rhs) => {
            let node_id = builder.add_node(format!("Binary: {}", binary_symbol(*op)));
            let left_id = add_expr(builder, lhs);
            let right_id = add_expr(builder, rhs);
            builder.add_edge(node_id, left_id);
            builder.add_edge(node_id, right_id);
            node_id
        }
        Expr::Call { callee, arguments } => {
            let node_id = builder.add_node("Call");
            let callee_id = add_expr(builder, callee);
            builder.add_edge(node_id, callee_id);
            for (idx, arg) in arguments.iter().enumerate() {
                let label = builder.add_node(format!("arg #{}", idx + 1));
                let arg_id = add_expr(builder, arg);
                builder.add_edge(node_id, label);
                builder.add_edge(label, arg_id);
            }
            node_id
        }
    }
}

fn describe_import(import: &Import) -> String {
    match &import.kind {
        ImportKind::Module { name, alias } => match alias {
            Some(alias) => format!("module {} as {}", name, alias),
            None => format!("module {}", name),
        },
        ImportKind::Symbol {
            module,
            symbol,
            alias,
        } => match alias {
            Some(alias) => format!("symbol {}::{} as {}", module, symbol, alias),
            None => format!("symbol {}::{}", module, symbol),
        },
    }
}

fn binary_symbol(op: BinaryOp) -> &'static str {
    match op {
        BinaryOp::Add => "+",
        BinaryOp::Sub => "-",
        BinaryOp::Mul => "*",
        BinaryOp::Div => "/",
        BinaryOp::Mod => "%",
        BinaryOp::Eq => "==",
        BinaryOp::Ne => "!=",
        BinaryOp::Lt => "<",
        BinaryOp::Le => "<=",
        BinaryOp::Gt => ">",
        BinaryOp::Ge => ">=",
        BinaryOp::And => "&&",
        BinaryOp::Or => "||",
    }
}

fn unary_symbol(op: UnaryOp) -> &'static str {
    match op {
        UnaryOp::Plus => "+",
        UnaryOp::Minus => "-",
        UnaryOp::Not => "!",
    }
}

#[derive(Default)]
struct DotBuilder {
    next_id: usize,
    nodes: Vec<String>,
    edges: Vec<String>,
}

impl DotBuilder {
    fn add_node(&mut self, label: impl AsRef<str>) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        let escaped = label.as_ref().replace('"', "\\\"");
        self.nodes.push(format!(
            "    n{idx} [label=\"{label}\"];",
            idx = id,
            label = escaped
        ));
        id
    }

    fn add_edge(&mut self, parent: usize, child: usize) {
        self.edges.push(format!(
            "    n{parent} -> n{child};",
            parent = parent,
            child = child
        ));
    }

    fn finish(self) -> String {
        let mut output =
            String::from("digraph ApexLangAST {\n    node [shape=box, fontname=\"FiraCode\"];\n");
        for node in self.nodes {
            output.push_str(&node);
            output.push('\n');
        }
        for edge in self.edges {
            output.push_str(&edge);
            output.push('\n');
        }
        output.push('}');
        output.push('\n');
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn visualize_simple_program() {
        let source = "fn helper(x) { return x; } fn apex() { return helper(1 + 2); }";
        let dot = visualize_source(source).expect("visualization to succeed");
        assert!(dot.contains("Program"));
        assert!(dot.contains("fn apex"));
        assert!(dot.contains("Binary: +"));
        assert!(dot.contains("Call"));
    }
}
