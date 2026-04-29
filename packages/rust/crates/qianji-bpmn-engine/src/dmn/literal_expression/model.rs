#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum NumericOperator {
    Add,
    Subtract,
}

pub(super) struct NumericPathExpression<'a> {
    pub(super) path: &'a str,
    pub(super) operator: NumericOperator,
    pub(super) rhs: f64,
}
