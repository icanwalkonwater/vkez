use std::path::PathBuf;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use proc_macro_error::{abort_if_dirty, emit_error, emit_warning, proc_macro_error};
use quote::quote;
use shaderc::{CompileOptions, EnvVersion, ShaderKind};
use structmeta::StructMeta;
use syn::{parse_macro_input, parse_quote, visit_mut::visit_item_mod_mut, ItemMod, LitStr};

use crate::shader_set::AccumulateShaderItemsVisitor;

mod shader_set;

#[proc_macro_error]
#[proc_macro_attribute]
pub fn shader_set(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut item = parse_macro_input!(item as ItemMod);
    let mut raw_attributes = AccumulateShaderItemsVisitor::default();
    visit_item_mod_mut(&mut raw_attributes, &mut item);

    for (shader, attributes) in &raw_attributes.shaders {
        dbg!(shader, attributes);
    }

    for (set, attributes) in &raw_attributes.descriptor_sets {
        dbg!(set, attributes);
    }

    todo!()
}

type ShaderModuleItem = ItemMod;

#[derive(StructMeta)]
struct ShaderModuleArgs {
    #[struct_meta(unnamed)]
    path: LitStr,
    kind: Option<LitStr>,
    entry: Option<LitStr>,
}

#[proc_macro_error]
#[proc_macro_attribute]
pub fn shader_module(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = parse_macro_input!(attr as ShaderModuleArgs);
    let item = parse_macro_input!(item as ShaderModuleItem);

    if let Some((_, content)) = item.content.as_ref() {
        if !content.is_empty() {
            emit_error!(item, "Expected module with empty body");
        }
    } else {
        emit_error!(item, "Expected module with empty body");
    }

    match shader_module_impl(attr, item) {
        Ok(tree) => tree.into(),
        Err(tree) => tree.into(),
    }
}

fn shader_module_impl(
    args: ShaderModuleArgs, item: ShaderModuleItem,
) -> Result<TokenStream2, TokenStream2> {
    let path = PathBuf::from(args.path.value());
    let absolute_path = if path.is_absolute() {
        path.clone()
    } else {
        std::env::current_dir().unwrap().join(&path)
    };

    let shader_source = std::fs::read_to_string(&path);
    if let Err(e) = &shader_source {
        if path.is_relative() {
            emit_error!(
                &args.path,
                format!("{e}");
                help = "Paths are relative to the crate's root"
            );
        }
    }
    let shader_source = shader_source.unwrap();

    let shader_kind = parse_shader_kind(args.kind.as_ref());

    abort_if_dirty();

    let compiler = shaderc::Compiler::new().unwrap();

    let entry_point = args
        .entry
        .as_ref()
        .map(|e| e.value())
        .unwrap_or("main".to_string());

    let mut compile_options = CompileOptions::new().unwrap();
    compile_options.set_target_env(shaderc::TargetEnv::Vulkan, EnvVersion::Vulkan1_1 as _);
    compile_options.set_optimization_level(shaderc::OptimizationLevel::Performance);

    let artifact = compiler
        .compile_into_spirv(
            &shader_source,
            shader_kind,
            &path.file_name().unwrap().to_string_lossy(),
            &entry_point,
            Some(&compile_options),
        )
        .map_err(|e| {
            syn::Error::new(args.path.span(), format!("Failed to compile shader: {e}"))
                .to_compile_error()
        })?;

    emit_warning!(&args.path, artifact.get_warning_messages());


    proc_macro_error::abort_if_dirty();

    let generated_module = gen_shader_module(
        &item,
        &absolute_path.to_string_lossy(),
        artifact.as_binary(),
    );
    Ok(quote!(#generated_module))
}

fn parse_shader_kind(kind: Option<&LitStr>) -> ShaderKind {
    if let Some(kind) = kind {
        match kind.value().as_str() {
            "Compute" => ShaderKind::Compute,
            _ => {
                emit_warning!(kind, "Unknown shader kind, defaulting to InferFromSource"; help = "See shaderc::ShaderKind");
                ShaderKind::InferFromSource
            }
        }
    } else {
        ShaderKind::InferFromSource
    }
}

fn gen_shader_module(original: &ItemMod, path: &str, code: &[u32]) -> ItemMod {
    let attrs = &original.attrs;
    let vis = &original.vis;
    let ident = &original.ident;

    let code_len = code.len();

    parse_quote! {
        #(#attrs)*
        #vis mod #ident {
            const _: &'static str = include_str!(#path);
            pub const CODE: [u32; #code_len] = [#(#code),*];
        }
    }
}
