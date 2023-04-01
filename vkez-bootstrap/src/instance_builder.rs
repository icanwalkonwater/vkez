use std::{
    borrow::{Borrow, Cow},
    ffi::{CStr, CString},
};

use ash::vk;
use vkez_core::ash;

use super::vkez_debug_utils_messenger;

#[derive(Default)]
pub struct InstanceBuilder<'a> {
    api_version: u32,
    app_name: Option<Cow<'a, CStr>>,
    app_version: u32,
    engine_name: Option<Cow<'a, CStr>>,
    engine_version: u32,
    extension_names: Vec<&'a CStr>,
    enable_debug_utils: bool,
}

impl<'builder> InstanceBuilder<'builder> {
    pub fn api_version(mut self, version: u32) -> Self {
        self.api_version = version;
        self
    }

    #[inline]
    pub fn app_name(mut self, name: impl AsRef<str>) -> Self {
        self.app_name = Some(CString::new(name.as_ref()).unwrap().into());
        self
    }

    #[inline]
    pub fn app_name_raw<'a: 'builder>(mut self, name: impl Into<Cow<'a, CStr>>) -> Self {
        self.app_name = Some(name.into());
        self
    }

    #[inline]
    pub fn app_version(mut self, version: u32) -> Self {
        self.app_version = version;
        self
    }

    #[inline]
    pub fn engine_name(mut self, name: impl AsRef<str>) -> Self {
        self.engine_name = Some(CString::new(name.as_ref()).unwrap().into());
        self
    }

    #[inline]
    pub fn engine_name_raw<'a: 'builder>(mut self, name: impl Into<Cow<'a, CStr>>) -> Self {
        self.engine_name = Some(name.into());
        self
    }

    #[inline]
    pub fn engine_version(mut self, version: u32) -> Self {
        self.engine_version = version;
        self
    }

    #[inline]
    pub fn enable_extension<'a: 'builder>(mut self, name: impl Into<&'a CStr>) -> Self {
        self.extension_names.push(name.into());
        self
    }

    #[inline]
    pub fn enable_extensions<'a: 'builder>(mut self, names: impl AsRef<[&'a CStr]>) -> Self {
        let names = names.as_ref();
        self.extension_names.extend_from_slice(names);
        self
    }

    #[inline]
    pub fn enable_default_debug_utils(mut self) -> Self {
        self.extension_names
            .push(ash::extensions::ext::DebugUtils::name());
        self.enable_debug_utils = true;
        self
    }

    #[inline]
    pub unsafe fn create_instance(
        self,
        entry: impl Borrow<ash::Entry>,
    ) -> ash::prelude::VkResult<(
        ash::Instance,
        Option<(ash::extensions::ext::DebugUtils, vk::DebugUtilsMessengerEXT)>,
    )> {
        let entry = entry.borrow();

        let mut debug_utils_messager_create_info = None;
        if self.enable_debug_utils {
            debug_utils_messager_create_info = Some(
                vk::DebugUtilsMessengerCreateInfoEXT::builder()
                    .message_severity(
                        vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
                            | vk::DebugUtilsMessageSeverityFlagsEXT::INFO
                            | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                            | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
                    )
                    .message_type(
                        vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                            | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
                            | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
                    )
                    .pfn_user_callback(Some(vkez_debug_utils_messenger)),
            );
        }

        let mut app_info = vk::ApplicationInfo::builder()
            .api_version(self.api_version)
            .application_version(self.app_version)
            .engine_version(self.engine_version);
        if let Some(app_name) = self.app_name.as_ref() {
            app_info = app_info.application_name(app_name);
        }
        if let Some(engine_name) = self.engine_name.as_ref() {
            app_info = app_info.engine_name(engine_name);
        }

        let extensions = self
            .extension_names
            .into_iter()
            .map(|n| n.as_ptr())
            .collect::<Vec<_>>();

        let mut create_info = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_extension_names(&extensions);

        if let Some(messenger) = debug_utils_messager_create_info.as_mut() {
            create_info = create_info.push_next(messenger);
        }

        let instance = entry.create_instance(&create_info, None)?;

        let mut debug_stuff = None;
        if let Some(messenger_create_info) = debug_utils_messager_create_info {
            let debug_utils = ash::extensions::ext::DebugUtils::new(entry, &instance);
            let messenger =
                debug_utils.create_debug_utils_messenger(&messenger_create_info, None)?;

            debug_stuff = Some((debug_utils, messenger));
        }

        Ok((instance, debug_stuff))
    }
}

pub trait AshInstanceExt {
    fn builder() -> InstanceBuilder<'static>;
}

impl AshInstanceExt for ash::Instance {
    fn builder() -> InstanceBuilder<'static> {
        InstanceBuilder::default()
    }
}
