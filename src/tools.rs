use std::collections::HashMap;

use crate::tools::{
    calculator::r#impl::CalculatorTool, file_delete::r#impl::DeleteFileTool,
    file_list::r#impl::ListFilesTool, file_read::r#impl::ReadFileTool,
    file_unzip::r#impl::UnzipFileTool, read_image::r#impl::ReadImageTool, tool::Tool,
};

pub mod calculator;
pub mod file_delete;
pub mod file_list;
pub mod file_read;
pub mod file_unzip;
pub mod read_image;
pub mod tool;

pub type ToolBox = HashMap<String, Box<dyn Tool>>;

pub fn build_toolbox() -> ToolBox {
    let tools: Vec<Box<dyn Tool>> = vec![Box::new(CalculatorTool)];

    into_toolbox(tools)
}

pub fn build_file_explorer_toolbox(vision_model: impl Into<String>) -> ToolBox {
    let tools: Vec<Box<dyn Tool>> = vec![
        Box::new(CalculatorTool),
        Box::new(UnzipFileTool),
        Box::new(ListFilesTool),
        Box::new(ReadFileTool),
        Box::new(ReadImageTool::new(vision_model)),
        Box::new(DeleteFileTool),
    ];

    into_toolbox(tools)
}

fn into_toolbox(tools: Vec<Box<dyn Tool>>) -> ToolBox {
    tools
        .into_iter()
        .map(|t| (t.name().to_string(), t))
        .collect()
}
