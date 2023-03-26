use std::path::PathBuf;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use proc_macro_error::{abort_if_dirty, emit_error, emit_warning, proc_macro_error};
use quote::quote;
use shaderc::ShaderKind;
use structmeta::StructMeta;
use syn::{parse_macro_input, parse_quote, ItemMod, LitStr};

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
    args: ShaderModuleArgs,
    item: ShaderModuleItem,
) -> Result<TokenStream2, TokenStream2> {
    let path = PathBuf::from(args.path.value());

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

    let artifact = compiler
        .compile_into_spirv(
            &shader_source,
            shader_kind,
            &path.file_name().unwrap().to_string_lossy(),
            &entry_point,
            None,
        )
        .map_err(|e| {
            syn::Error::new(args.path.span(), format!("Failed to compile shader: {e}"))
                .to_compile_error()
        })?;

    emit_warning!(&args.path, artifact.get_warning_messages());

    let generated_module = gen_shader_module(&item, artifact.as_binary());
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

fn gen_shader_module(original: &ItemMod, code: &[u32]) -> ItemMod {
    let attrs = &original.attrs;
    let vis = &original.vis;
    let ident = &original.ident;

    let code_len = code.len();

    parse_quote! {
        #(#attrs)*
        #vis mod #ident {
            pub const CODE: [u32; #code_len] = [#(#code),*];
        }
    }
}
