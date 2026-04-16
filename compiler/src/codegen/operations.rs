use inkwell::values::AnyValue;

use super::{CodeGen, CodeGenError};
use crate::{codegen::data::GenValue, semantics::{Ownership, SemanticExpression}, tokens::ComparisonType};

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

impl From<ComparisonType> for inkwell::FloatPredicate {
    fn from(op: ComparisonType) -> Self {
        match op {
            ComparisonType::Equal => inkwell::FloatPredicate::OEQ,
            ComparisonType::NotEqual => inkwell::FloatPredicate::ONE,
            ComparisonType::GreaterThan => inkwell::FloatPredicate::OGT,
            ComparisonType::LessThan => inkwell::FloatPredicate::OLT,
            ComparisonType::GreaterThanOrEqual => inkwell::FloatPredicate::OGE,
            ComparisonType::LessThanOrEqual => inkwell::FloatPredicate::OLE,
        }
    }
}

impl<'ctxt> CodeGen<'ctxt> {
    pub fn gen_add(&mut self, expr1: &SemanticExpression, expr2: &SemanticExpression) -> Result<GenValue<'ctxt>, CodeGenError> {
        let val1 = self.gen_eval(expr1)?;
        let val2 = self.gen_eval(expr2)?;
        if let (GenValue::Integer(int1), GenValue::Integer(int2)) = (&val1, &val2) {
            let res = self.builder.build_int_add(*int1, *int2, "sum")?;
            Ok(GenValue::Integer(res))
        } else if let (GenValue::Float(float1), GenValue::Float(float2)) = (&val1, &val2) {
            let res = self.builder.build_float_add(*float1, *float2, "sumf")?;
            Ok(GenValue::Float(res))
        } else if let (GenValue::String { value: str1, .. }, GenValue::String { value: str2, .. })
            = (&val1, &val2) 
        {
            let res = self.builder.build_call(
                self.runtime.concat_string,
                &[(*str1).into(), (*str2).into()],
                "str_concat"
            )?.as_any_value_enum().into_pointer_value();

            self.remove_if_owned(val1, &expr1.sem_type)?;
            self.remove_if_owned(val2, &expr2.sem_type)?;

            Ok(GenValue::String {
                value: res, ownership:
                Ownership::Owned
            })
        } else {
            panic!("Unexpected types for addition");
        }
    }

    pub fn gen_subtract(&mut self, expr1: &SemanticExpression, expr2: &SemanticExpression) -> Result<GenValue<'ctxt>, CodeGenError> {
        let val1 = self.gen_eval(expr1)?;
        let val2 = self.gen_eval(expr2)?;
        if let (GenValue::Integer(int1), GenValue::Integer(int2)) = (&val1, &val2) {
            let res = self.builder.build_int_sub(*int1, *int2, "sub")?;
            Ok(GenValue::Integer(res))
        } else if let (GenValue::Float(float1), GenValue::Float(float2)) = (&val1, &val2) {
            let res = self.builder.build_float_sub(*float1, *float2, "subf")?;
            Ok(GenValue::Float(res))
        } else {
            panic!("Unexpected types for subtraction");
        }
    }

    pub fn gen_multiply(&mut self, expr1: &SemanticExpression, expr2: &SemanticExpression) -> Result<GenValue<'ctxt>, CodeGenError> {
        let val1 = self.gen_eval(expr1)?;
        let val2 = self.gen_eval(expr2)?;
        if let (GenValue::Integer(int1), GenValue::Integer(int2)) = (&val1, &val2) {
            let res = self.builder.build_int_mul(*int1, *int2, "mul")?;
            Ok(GenValue::Integer(res))
        } else if let (GenValue::Float(float1), GenValue::Float(float2)) = (&val1, &val2) {
            let res = self.builder.build_float_mul(*float1, *float2, "mulf")?;
            Ok(GenValue::Float(res))
        } else {
            panic!("Unexpected types for multiplication");
        }
    }

    pub fn gen_divide(&mut self, expr1: &SemanticExpression, expr2: &SemanticExpression) -> Result<GenValue<'ctxt>, CodeGenError> {
        let val1 = self.gen_eval(expr1)?;
        let val2 = self.gen_eval(expr2)?;
        if let (GenValue::Integer(int1), GenValue::Integer(int2)) = (&val1, &val2) {
            let res = self.builder.build_int_signed_div(*int1, *int2, "div")?;
            Ok(GenValue::Integer(res))
        } else if let (GenValue::Float(float1), GenValue::Float(float2)) = (&val1, &val2) {
            let res = self.builder.build_float_div(*float1, *float2, "divf")?;
            Ok(GenValue::Float(res))
        } else {
            panic!("Unexpected types for division");
        }
    }

    pub fn gen_modulus(&mut self, expr1: &SemanticExpression, expr2: &SemanticExpression) -> Result<GenValue<'ctxt>, CodeGenError> {
        let val1 = self.gen_eval(expr1)?;
        let val2 = self.gen_eval(expr2)?;
        if let (GenValue::Integer(int1), GenValue::Integer(int2)) = (&val1, &val2) {
            let res = self.builder.build_int_signed_rem(*int1, *int2, "mod")?;
            Ok(GenValue::Integer(res))
        } else {
            panic!("Unexpected types for modulus");
        }
    }

    pub fn gen_logical_and(&mut self, expr1: &SemanticExpression, expr2: &SemanticExpression) -> Result<GenValue<'ctxt>, CodeGenError> {
        let val1 = self.gen_eval(expr1)?;
        let val2 = self.gen_eval(expr2)?;
        if let (GenValue::Bool(bool1), GenValue::Bool(bool2)) = (&val1, &val2) {
            let res = self.builder.build_and(*bool1, *bool2, "and")?;
            Ok(GenValue::Bool(res))
        } else {
            panic!("Unexpected types for logical and");
        }
    }

    pub fn gen_logical_or(&mut self, expr1: &SemanticExpression, expr2: &SemanticExpression) -> Result<GenValue<'ctxt>, CodeGenError> {
        let val1 = self.gen_eval(expr1)?;
        let val2 = self.gen_eval(expr2)?;
        if let (GenValue::Bool(bool1), GenValue::Bool(bool2)) = (&val1, &val2) {
            let res = self.builder.build_or(*bool1, *bool2, "or")?;
            Ok(GenValue::Bool(res))
        } else {
            panic!("Unexpected types for logical or");
        }
    }

    pub fn gen_logical_not(&mut self, expr: &SemanticExpression) -> Result<GenValue<'ctxt>, CodeGenError> {
        let val = self.gen_eval(expr)?;
        if let GenValue::Bool(bool_val) = val {
            let res = self.builder.build_not(bool_val, "not")?;
            Ok(GenValue::Bool(res))
        } else {
            panic!("Unexpected type for logical not");
        }
    }

    pub fn gen_compare(&mut self, expr1: &SemanticExpression, expr2: &SemanticExpression, op: ComparisonType) -> Result<GenValue<'ctxt>, CodeGenError> {
        let val1 = self.gen_eval(expr1)?;
        let val2 = self.gen_eval(expr2)?;
        if let (GenValue::Integer(int1), GenValue::Integer(int2)) = (&val1, &val2) {
            let res = self.builder.build_int_compare(op.into(), *int1, *int2, "cmp")?;
            Ok(GenValue::Bool(res))
        } else if let (GenValue::Float(float1), GenValue::Float(float2)) = (&val1, &val2) {
            let res = self.builder.build_float_compare(op.into(), *float1, *float2, "cmpf")?;
            Ok(GenValue::Bool(res))
        } else if let (GenValue::String { value: str1, .. }, GenValue::String { value: str2, .. })
            = (&val1, &val2) 
        {
            let res = self.builder.build_call(
                self.runtime.compare_string,
                &[(*str1).into(), (*str2).into()],
                "str_compare"
            )?.as_any_value_enum().into_int_value();
            let cmp = self.builder.build_int_compare(op.into(), res, self.int_type().const_zero(), "str_cmp")?;

            self.remove_if_owned(val1, &expr1.sem_type)?;
            self.remove_if_owned(val2, &expr2.sem_type)?;

            Ok(GenValue::Bool(cmp))
        } else {
            panic!("Unexpected types for comparison");
        }
    }
}
