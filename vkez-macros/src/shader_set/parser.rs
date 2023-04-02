use std::ops::BitOr;

use proc_macro_error::emit_error;
use structmeta::StructMeta;
use syn::{spanned::Spanned, visit_mut::VisitMut, Ident, LitStr, Path};

#[derive(StructMeta, Default, Debug)]
pub(crate) struct ShaderSetShaderAttributes {
    pub file: Option<LitStr>,
    pub lang: Option<LitStr>,
    pub kind: Option<Ident>,
}

impl BitOr for ShaderSetShaderAttributes {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        // FIXME: handle both sides having a value ?
        Self {
            file: self.file.or(rhs.file),
            lang: self.lang.or(rhs.lang),
            kind: self.kind.or(rhs.kind),
        }
    }
}

#[derive(StructMeta, Default, Debug)]
pub(crate) struct ShaderSetDescriptorSetAttributes {
    pub from_shader: Option<Path>,
}

impl BitOr for ShaderSetDescriptorSetAttributes {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        // FIXME: handle both sides having a value ?
        Self {
            from_shader: self.from_shader.or(rhs.from_shader),
        }
    }
}

#[derive(Default, Debug)]
pub(crate) struct AccumulateShaderItemsVisitor {
    pub shaders: Vec<(Ident, ShaderSetShaderAttributes)>,
    pub descriptor_sets: Vec<(Ident, ShaderSetDescriptorSetAttributes)>,
}

impl VisitMut for AccumulateShaderItemsVisitor {
    fn visit_item_struct_mut(&mut self, item: &mut syn::ItemStruct) {
        let mut shader_attrs = Vec::new();
        let mut descriptor_set_attrs = Vec::new();

        // Collect interresting attributes
        // Iterate attributes in reverse order so that when we remove some items, the
        // indices remain valids
        for i in (0..item.attrs.len()).rev() {
            let attr = &item.attrs[i];
            let path = attr.meta.path();

            // Do we what to collect this attribute and where
            let to_container = match path {
                _ if path.is_ident("shader") => Some(&mut shader_attrs),
                _ if path.is_ident("descriptor_set") => Some(&mut descriptor_set_attrs),
                _ => None,
            };

            // Collect it
            if let Some(to_container) = to_container {
                to_container.push(item.attrs.remove(i));
            }
        }

        // Check for too many attributes
        let has_shader_attrs = !shader_attrs.is_empty();
        let has_descriptor_set_attrs = !descriptor_set_attrs.is_empty();

        if has_shader_attrs && has_descriptor_set_attrs {
            let first_shader_attr = shader_attrs.first().unwrap();
            let first_descriptor_set_attr = descriptor_set_attrs.first().unwrap();
            emit_error!(
                item.span(), "Item has mutually exclusive attributes";
                help = first_shader_attr.span() => "Found shader only attribute";
                help = first_descriptor_set_attr.span() => "Found descriptor set only attribute";
            );
        }

        if has_shader_attrs {
            self.shaders.push((
                item.ident.clone(),
                shader_attrs
                    .into_iter()
                    .fold(ShaderSetShaderAttributes::default(), |attributes, attr| {
                        attributes | attr.parse_args().unwrap()
                    }),
            ));
        } else if has_descriptor_set_attrs {
            self.descriptor_sets.push((
                item.ident.clone(),
                descriptor_set_attrs.into_iter().fold(
                    ShaderSetDescriptorSetAttributes::default(),
                    |attribtes, attr| attribtes | attr.parse_args().unwrap(),
                ),
            ));
        }
    }
}
