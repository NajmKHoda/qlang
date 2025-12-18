use super::{CodeGen, CodeGenError, QLValue};

pub enum ComparisonOp {
    Equal,
    NotEqual,
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual
}

impl From<ComparisonOp> for inkwell::IntPredicate {
    fn from(op: ComparisonOp) -> Self {
        match op {
            ComparisonOp::Equal => inkwell::IntPredicate::EQ,
            ComparisonOp::NotEqual => inkwell::IntPredicate::NE,
            ComparisonOp::GreaterThan => inkwell::IntPredicate::SGT,
            ComparisonOp::LessThan => inkwell::IntPredicate::SLT,
            ComparisonOp::GreaterThanOrEqual => inkwell::IntPredicate::SGE,
            ComparisonOp::LessThanOrEqual => inkwell::IntPredicate::SLE,
        }
    }
}

impl<'ctxt> CodeGen<'ctxt> {
    pub fn gen_add(&self, val1: QLValue<'ctxt>, val2: QLValue<'ctxt>) -> Result<QLValue<'ctxt>, CodeGenError> {
        if let (QLValue::Integer(int1), QLValue::Integer(int2)) = (val1, val2) {
            let res = self.builder.build_int_add(int1, int2, "sum")?;
            Ok(QLValue::Integer(res))
        } else {
            Err(CodeGenError::UnexpectedTypeError)
        }
    }

    pub fn gen_subtract(&self, val1: QLValue<'ctxt>, val2: QLValue<'ctxt>) -> Result<QLValue<'ctxt>, CodeGenError> {
        if let (QLValue::Integer(int1), QLValue::Integer(int2)) = (val1, val2) {
            let res = self.builder.build_int_sub(int1, int2, "sub")?;
            Ok(QLValue::Integer(res))
        } else {
            Err(CodeGenError::UnexpectedTypeError)
        }
    }

    pub fn gen_compare(&self, val1: QLValue<'ctxt>, val2: QLValue<'ctxt>, op: ComparisonOp) -> Result<QLValue<'ctxt>, CodeGenError> {
        if let (QLValue::Integer(int1), QLValue::Integer(int2)) = (val1, val2) {
            let res = self.builder.build_int_compare(op.into(), int1, int2, "cmp")?;
            Ok(QLValue::Bool(res))
        } else {
            Err(CodeGenError::UnexpectedTypeError)
        }
    }
}
