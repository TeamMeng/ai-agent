use std::collections::HashMap;

use crate::tools::{calculator::r#impl::CalculatorTool, tool::Tool};

pub mod calculator;
pub mod tool;

pub type ToolBox = HashMap<String, Box<dyn Tool>>;

pub fn build_toolbox() -> ToolBox {
    let tools: Vec<Box<dyn Tool>> = vec![Box::new(CalculatorTool)];

    tools
        .into_iter()
        .map(|t| (t.name().to_string(), t))
        .collect()
}
