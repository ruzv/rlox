struct Binary<Expr> {
    left: Box<Expr>,
    operator: i64,
    right: Box<Expr>,
}
