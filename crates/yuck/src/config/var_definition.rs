use simplexpr::dynval::DynVal;

use crate::{
    error::{DiagResult, DiagResultExt},
    parser::{ast::Ast, ast_iterator::AstIterator, from_ast::FromAstElementContent},
};
use eww_shared_util::{Span, VarName};

#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize)]
pub struct VarDefinition {
    pub name: VarName,
    pub initial_value: DynVal,
    pub per_window: bool,
    pub span: Span,
}

impl FromAstElementContent for VarDefinition {
    const ELEMENT_NAME: &'static str = "defvar";

    fn from_tail<I: Iterator<Item = Ast>>(span: Span, mut iter: AstIterator<I>) -> DiagResult<Self> {
        let result: DiagResult<_> = try {
            let (_, name) = iter.expect_symbol()?;
            let mut attrs = iter.expect_key_values()?;
            let per_window: bool = attrs.primitive_optional("per_window")?.unwrap_or(false);
            let (_, initial_value) = iter.expect_literal()?;
            iter.expect_done()?;
            Self { name: VarName(name), per_window, initial_value, span }
        };
        result.note(r#"Expected format: `(defvar name "initial-value")`"#)
    }
}
