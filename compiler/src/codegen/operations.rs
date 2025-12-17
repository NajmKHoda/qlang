use super::{CodeGen, CodeGenError, QLValue};

impl<'ctxt> CodeGen<'ctxt> {
    pub fn add(&self, val1: QLValue<'ctxt>, val2: QLValue<'ctxt>) -> Result<QLValue<'ctxt>, CodeGenError> {
        if let (QLValue::Integer(int1), QLValue::Integer(int2)) = (val1, val2) {
            let res = self.builder.build_int_add(int1, int2, "sum")?;
            Ok(QLValue::Integer(res))
        } else {
            Err(CodeGenError::UnexpectedTypeError)
        }
    }

    pub fn eq(&self, val1: QLValue<'ctxt>, val2: QLValue<'ctxt>) -> Result<QLValue<'ctxt>, CodeGenError> {
        if let (QLValue::Integer(int1), QLValue::Integer(int2)) = (val1, val2) {
            let res = self.builder.build_int_compare(inkwell::IntPredicate::EQ, int1, int2, "eq")?;
            Ok(QLValue::Bool(res))
        } else {
            Err(CodeGenError::UnexpectedTypeError)
        }
    }
}
