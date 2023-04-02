mod parser;

pub(crate) use parser::*;

// pub(crate) struct ShaderState {
//     pub artifact: CompilationArtifact,
//     pub reflect: naga::Module,
// }

// pub(crate) struct ShaderSetState<'a> {
//     pub compiler: shaderc::Compiler,
//     pub base_compiler_options: CompileOptions<'a>,
//     pub shaders: HashMap<Ident, ShaderState>,
// }
