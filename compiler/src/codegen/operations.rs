use inkwell::values::AnyValue;

use super::{CodeGen, CodeGenError, QLValue};

#[derive(Clone, Copy, Debug)]
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
        if let (QLValue::Integer(int1), QLValue::Integer(int2)) = (&val1, &val2) {
            let res = self.builder.build_int_add(*int1, *int2, "sum")?;
            Ok(QLValue::Integer(res))
        } else if let (QLValue::String(str1, _), QLValue::String(str2, _)) = (&val1, &val2) {
            let res = self.builder.build_call(
                self.runtime_functions.concat_string.into(),
                &[(*str1).into(), (*str2).into()],
                "str_concat"
            )?.as_any_value_enum().into_pointer_value();

            self.remove_if_temp(val1)?;
            self.remove_if_temp(val2)?;

            Ok(QLValue::String(res, false))
        } else {
            Err(CodeGenError::UnexpectedTypeError)
        }
    }

    pub fn gen_subtract(&self, val1: QLValue<'ctxt>, val2: QLValue<'ctxt>) -> Result<QLValue<'ctxt>, CodeGenError> {
        if let (QLValue::Integer(int1), QLValue::Integer(int2)) = (&val1, &val2) {
            let res = self.builder.build_int_sub(*int1, *int2, "sub")?;
            Ok(QLValue::Integer(res))
        } else {
            Err(CodeGenError::UnexpectedTypeError)
        }
    }

    pub fn gen_compare(&self, val1: QLValue<'ctxt>, val2: QLValue<'ctxt>, op: ComparisonOp) -> Result<QLValue<'ctxt>, CodeGenError> {
        if let (QLValue::Integer(int1), QLValue::Integer(int2)) = (&val1, &val2) {
            let res = self.builder.build_int_compare(op.into(), *int1, *int2, "cmp")?;
            Ok(QLValue::Bool(res))
        } else if let (QLValue::String(str1, _), QLValue::String(str2, _)) = (&val1, &val2) {
            let res = self.builder.build_call(
                self.runtime_functions.compare_string.into(),
                &[(*str1).into(), (*str2).into()],
                "str_compare"
            )?.as_any_value_enum().into_int_value();
            let cmp = self.builder.build_int_compare(op.into(), res, self.int_type().const_zero(), "str_cmp")?;

            self.remove_if_temp(val1)?;
            self.remove_if_temp(val2)?;

            Ok(QLValue::Bool(cmp))
        } else {
            Err(CodeGenError::UnexpectedTypeError)
        }
    }
}
