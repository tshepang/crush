use crate::lang::value::Value;
use crate::lang::scope::Scope;
use crate::lang::errors::CrushResult;

pub fn declare(root: &Scope) -> CrushResult<()> {
    let env = root.create_namespace("constants")?;
    root.r#use(&env);
    env.declare("true", Value::Bool(true))?;
    env.declare("false", Value::Bool(false))?;
    env.declare("global", Value::Scope(root.clone()))?;
    env.readonly();
    Ok(())
}
