use crate::StmtId;

#[derive(Clone, Debug, PartialEq)]
pub struct BlockStmt {
    pub stmts: Vec<StmtId>,
}
