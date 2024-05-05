use crate::value::{Number, Primitive, Value, ValueError};

/// Calculate the tangent of the top value on the stack
/// Expects a value in radians
pub fn tan(input: Value) -> Result<Value, ValueError> {
    let v = input.cast_decimal()?.into_f64();
    let v = v.tan();
    Ok(Value::Primitive(Primitive::Decimal(Number::from_f64(v)?)))
}

/// Calculate the sine of the top value on the stack
/// Expects a value in radians
pub fn sin(input: Value) -> Result<Value, ValueError> {
    let v = input.cast_decimal()?.into_f64();
    let v = v.sin();
    Ok(Value::Primitive(Primitive::Decimal(Number::from_f64(v)?)))
}

/// Calculate the cosine of the top value on the stack
/// Expects a value in radians
pub fn cos(input: Value) -> Result<Value, ValueError> {
    let v = input.cast_decimal()?.into_f64();
    let v = v.cos();
    Ok(Value::Primitive(Primitive::Decimal(Number::from_f64(v)?)))
}

/// Calculate the arctangent2 of the top value on the stack
/// Consumes 2 stack values; [Y, X]
pub fn atan2(y: Value, x: Value) -> Result<Value, ValueError> {
    let y = y.cast_decimal()?.into_f64();
    let x = x.cast_decimal()?.into_f64();
    let v = y.atan2(x);
    Ok(Value::Primitive(Primitive::Decimal(Number::from_f64(v)?)))
}

/// Calculate the arctangent of the top value on the stack
pub fn atan(input: Value) -> Result<Value, ValueError> {
    let v = input.cast_decimal()?.into_f64();
    let v = v.atan();
    Ok(Value::Primitive(Primitive::Decimal(Number::from_f64(v)?)))
}

/// Calculate the arcsine of the top value on the stack
pub fn asin(input: Value) -> Result<Value, ValueError> {
    let v = input.cast_decimal()?.into_f64();
    let v = v.asin();
    Ok(Value::Primitive(Primitive::Decimal(Number::from_f64(v)?)))
}

/// Calculate the arccosine of the top value on the stack
pub fn acos(input: Value) -> Result<Value, ValueError> {
    let v = input.cast_decimal()?.into_f64();
    let v = v.acos();
    Ok(Value::Primitive(Primitive::Decimal(Number::from_f64(v)?)))
}

/// Calculate the hyperbolic tangent of the top value on the stack
pub fn tanh(input: Value) -> Result<Value, ValueError> {
    let v = input.cast_decimal()?.into_f64();
    let v = v.tanh();
    Ok(Value::Primitive(Primitive::Decimal(Number::from_f64(v)?)))
}

/// Calculate the hyperbolic sine of the top value on the stack
pub fn sinh(input: Value) -> Result<Value, ValueError> {
    let v = input.cast_decimal()?.into_f64();
    let v = v.sinh();
    Ok(Value::Primitive(Primitive::Decimal(Number::from_f64(v)?)))
}

/// Calculate the hyperbolic cosine of the top value on the stack
pub fn cosh(input: Value) -> Result<Value, ValueError> {
    let v = input.cast_decimal()?.into_f64();
    let v = v.cosh();
    Ok(Value::Primitive(Primitive::Decimal(Number::from_f64(v)?)))
}

pub fn round(input: Value, precision: Value) -> Result<Value, ValueError> {
    let v = input.cast_decimal()?;
    let precision = precision.cast_integer()?;

    let v = v.round(precision as i8)?;
    Ok(Value::Primitive(Primitive::Decimal(v)))
}

pub fn log(input: Value, base: Value) -> Result<Value, ValueError> {
    let v = input.cast_decimal()?.into_f64();
    let base = base.cast_decimal()?.into_f64();
    let v = v.log(base);
    Ok(Value::Primitive(Primitive::Decimal(Number::from_f64(v)?)))
}

pub fn ilog(input: Value, base: Value) -> Result<Value, ValueError> {
    let v = input.cast_integer()?;
    let base = base.cast_integer()?;
    let v = v.ilog(base);
    Ok(Value::Primitive(Primitive::Integer(v as i128)))
}

pub fn root(input: Value, n: Value) -> Result<Value, ValueError> {
    let v = input.cast_decimal()?.into_f64();
    let n = n.cast_decimal()?.into_f64();
    let v = n.powf(1.0 / v);
    Ok(Value::Primitive(Primitive::Decimal(Number::from_f64(v)?)))
}
