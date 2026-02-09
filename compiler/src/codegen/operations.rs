use inkwell::values::AnyValue;

use super::{CodeGen, CodeGenError};
use crate::{codegen::data::GenValue, semantics::Ownership, tokens::ComparisonType};

impl From<ComparisonType> for inkwell::IntPredicate {
    fn from(op: ComparisonType) -> Self {
        match op {
            ComparisonType::Equal => inkwell::IntPredicate::EQ,
            ComparisonType::NotEqual => inkwell::IntPredicate::NE,
            ComparisonType::GreaterThan => inkwell::IntPredicate::SGT,
            ComparisonType::LessThan => inkwell::IntPredicate::SLT,
            ComparisonType::GreaterThanOrEqual => inkwell::IntPredicate::SGE,
            ComparisonType::LessThanOrEqual => inkwell::IntPredicate::SLE,
        }
    }
}

impl<'ctxt> CodeGen<'ctxt> {
    pub fn gen_add(&self, val1: GenValue<'ctxt>, val2: GenValue<'ctxt>) -> Result<GenValue<'ctxt>, CodeGenError> {
        if let (GenValue::Integer(int1), GenValue::Integer(int2)) = (&val1, &val2) {
            let res = self.builder.build_int_add(*int1, *int2, "sum")?;
            Ok(GenValue::Integer(res))
        } else if let (GenValue::String { value: str1, .. }, GenValue::String { value: str2, .. })
            = (&val1, &val2) 
        {
            let res = self.builder.build_call(
                self.runtime_functions.concat_string,
                &[(*str1).into(), (*str2).into()],
                "str_concat"
            )?.as_any_value_enum().into_pointer_value();

            self.remove_if_owned(val1)?;
            self.remove_if_owned(val2)?;

            Ok(GenValue::String {
                value: res, ownership:
                Ownership::Owned
            })
        } else {
            panic!("Unexpected types for addition");
        }
    }

    pub fn gen_subtract(&self, val1: GenValue<'ctxt>, val2: GenValue<'ctxt>) -> Result<GenValue<'ctxt>, CodeGenError> {
        if let (GenValue::Integer(int1), GenValue::Integer(int2)) = (&val1, &val2) {
            let res = self.builder.build_int_sub(*int1, *int2, "sub")?;
            Ok(GenValue::Integer(res))
        } else {
            panic!("Unexpected types for subtraction");
        }
    }

    pub fn gen_compare(&self, val1: GenValue<'ctxt>, val2: GenValue<'ctxt>, op: ComparisonType) -> Result<GenValue<'ctxt>, CodeGenError> {
        if let (GenValue::Integer(int1), GenValue::Integer(int2)) = (&val1, &val2) {
            let res = self.builder.build_int_compare(op.into(), *int1, *int2, "cmp")?;
            Ok(GenValue::Bool(res))
        } else if let (GenValue::String { value: str1, .. }, GenValue::String { value: str2, .. })
            = (&val1, &val2) 
        {
            let res = self.builder.build_call(
                self.runtime_functions.compare_string,
                &[(*str1).into(), (*str2).into()],
                "str_compare"
            )?.as_any_value_enum().into_int_value();
            let cmp = self.builder.build_int_compare(op.into(), res, self.int_type().const_zero(), "str_cmp")?;

            self.remove_if_owned(val1)?;
            self.remove_if_owned(val2)?;

            Ok(GenValue::Bool(cmp))
        } else {
            panic!("Unexpected types for comparison");
        }
    }
}
