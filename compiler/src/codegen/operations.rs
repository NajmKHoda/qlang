use inkwell::values::AnyValue;

use super::{CodeGen, CodeGenError};
use crate::{codegen::data::GenValue, semantics::{Ownership, SemanticExpression, SemanticType, SemanticTypeKind}, tokens::ComparisonType};

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
    pub fn gen_convert(&mut self, expr: &SemanticExpression, target_type: &SemanticType) -> Result<GenValue<'ctxt>, CodeGenError> {
        let source_val = self.gen_eval(expr)?;
        let source_kind = expr.sem_type.kind();
        let target_kind = target_type.kind();

        if source_kind == target_kind {
            return Ok(source_val);
        }

        match (&source_kind, &target_kind) {
            (SemanticTypeKind::Integer, SemanticTypeKind::String) => {
                let GenValue::Integer(int_val) = source_val else { panic!("Expected integer"); };
                let str_val = self.builder.build_call(
                    self.runtime.int_to_string,
                    &[int_val.into()],
                    "int_to_string"
                )?.as_any_value_enum().into_pointer_value();
                Ok(GenValue::String { value: str_val, ownership: Ownership::Owned })
            }
            (SemanticTypeKind::String, SemanticTypeKind::Integer) => {
                let GenValue::String { value: str_val, .. } = source_val.clone() else { panic!("Expected string"); };
                let int_val = self.builder.build_call(
                    self.runtime.str_to_int,
                    &[str_val.into()],
                    "str_to_int"
                )?.as_any_value_enum().into_int_value();
                self.remove_if_owned(source_val, &expr.sem_type)?;
                Ok(GenValue::Integer(int_val))
            }
            (SemanticTypeKind::Integer, SemanticTypeKind::Float) => {
                let GenValue::Integer(int_val) = source_val else { panic!("Expected integer"); };
                let float_val = self.builder.build_signed_int_to_float(int_val, self.float_type(), "int_to_float")?;
                Ok(GenValue::Float(float_val))
            }
            (SemanticTypeKind::Float, SemanticTypeKind::Integer) => {
                let GenValue::Float(float_val) = source_val else { panic!("Expected float"); };
                let int_val = self.builder.build_call(
                    self.runtime.float_to_int,
                    &[float_val.into()],
                    "float_to_int"
                )?.as_any_value_enum().into_int_value();
                Ok(GenValue::Integer(int_val))
            }
            (SemanticTypeKind::Integer, SemanticTypeKind::Bool) => {
                let GenValue::Integer(int_val) = source_val else { panic!("Expected integer"); };
                let bool_val = self.builder.build_int_compare(
                    inkwell::IntPredicate::NE,
                    int_val,
                    self.int_type().const_zero(),
                    "int_to_bool"
                )?;
                Ok(GenValue::Bool(bool_val))
            }
            (SemanticTypeKind::Bool, SemanticTypeKind::Integer) => {
                let GenValue::Bool(bool_val) = source_val else { panic!("Expected bool"); };
                let int_val = self.builder.build_int_z_extend(bool_val, self.int_type(), "bool_to_int")?;
                Ok(GenValue::Integer(int_val))
            }
            (SemanticTypeKind::Float, SemanticTypeKind::String) => {
                let GenValue::Float(float_val) = source_val else { panic!("Expected float"); };
                let str_val = self.builder.build_call(
                    self.runtime.float_to_string,
                    &[float_val.into()],
                    "float_to_string"
                )?.as_any_value_enum().into_pointer_value();
                Ok(GenValue::String { value: str_val, ownership: Ownership::Owned })
            }
            (SemanticTypeKind::String, SemanticTypeKind::Float) => {
                let GenValue::String { value: str_val, .. } = source_val.clone() else { panic!("Expected string"); };
                let float_val = self.builder.build_call(
                    self.runtime.str_to_float,
                    &[str_val.into()],
                    "str_to_float"
                )?.as_any_value_enum().into_float_value();
                self.remove_if_owned(source_val, &expr.sem_type)?;
                Ok(GenValue::Float(float_val))
            }
            (SemanticTypeKind::Float, SemanticTypeKind::Bool) => {
                let GenValue::Float(float_val) = source_val else { panic!("Expected float"); };
                let bool_val = self.builder.build_float_compare(
                    inkwell::FloatPredicate::ONE,
                    float_val,
                    self.float_type().const_float(0.0),
                    "float_to_bool"
                )?;
                Ok(GenValue::Bool(bool_val))
            }
            (SemanticTypeKind::Bool, SemanticTypeKind::Float) => {
                let GenValue::Bool(bool_val) = source_val else { panic!("Expected bool"); };
                let float_val = self.builder.build_unsigned_int_to_float(bool_val, self.float_type(), "bool_to_float")?;
                Ok(GenValue::Float(float_val))
            }
            (SemanticTypeKind::Bool, SemanticTypeKind::String) => {
                let GenValue::Bool(bool_val) = source_val else { panic!("Expected bool"); };
                let str_val = self.builder.build_call(
                    self.runtime.bool_to_string,
                    &[bool_val.into()],
                    "bool_to_string"
                )?.as_any_value_enum().into_pointer_value();
                Ok(GenValue::String { value: str_val, ownership: Ownership::Owned })
            }
            (SemanticTypeKind::String, SemanticTypeKind::Bool) => {
                let GenValue::String { value: str_val, .. } = source_val.clone() else { panic!("Expected string"); };
                let bool_val = self.builder.build_call(
                    self.runtime.str_to_bool,
                    &[str_val.into()],
                    "str_to_bool"
                )?.as_any_value_enum().into_int_value();
                self.remove_if_owned(source_val, &expr.sem_type)?;
                Ok(GenValue::Bool(bool_val))
            }
            _ => panic!("Unsupported conversion in codegen"),
        }
    }

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
