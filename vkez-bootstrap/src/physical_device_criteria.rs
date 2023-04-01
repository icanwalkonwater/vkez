use std::{borrow::Cow, ffi::CStr};

use ash::vk;
use vkez_core::ash;
use vkez_core::tracing;

use super::QueueFamilyRequest;

#[derive(Debug, Clone)]
pub struct PhysicalDeviceMetadata {
    pub handle: vk::PhysicalDevice,
    pub features: vk::PhysicalDeviceFeatures,
    pub properties: vk::PhysicalDeviceProperties,
    pub extensions: Vec<vk::ExtensionProperties>,
    pub queue_families: Vec<vk::QueueFamilyProperties>,
}

#[derive(Debug, Default, Clone)]
pub struct PhysicalDeviceCriteria<'a> {
    pub device_type_preference: Vec<vk::PhysicalDeviceType>,
    pub queue_families: Vec<Cow<'a, QueueFamilyRequest>>,
    pub minimum_api_version: u32,
    pub required_extensions: Vec<Cow<'a, CStr>>,
    // pub prefered_extensions: Vec<Cow<'a, CStr>>,
}

impl<'crit> PhysicalDeviceCriteria<'crit> {
    pub fn empty() -> Self {
        Self::default()
    }

    /// Earlier calls will have the most precedence
    pub fn prefer_device_type(mut self, ty: vk::PhysicalDeviceType) -> Self {
        self.device_type_preference.push(ty);
        self
    }

    pub fn request_queue_family<'a: 'crit>(
        mut self,
        queue: impl Into<Cow<'a, QueueFamilyRequest>>,
    ) -> Self {
        self.queue_families.push(queue.into());
        self
    }

    pub fn require_extension<'a: 'crit>(mut self, name: impl Into<Cow<'a, CStr>>) -> Self {
        self.required_extensions.push(name.into());
        self
    }

    pub fn minimum_api_version(mut self, version: u32) -> Self {
        self.minimum_api_version = version;
        self
    }

    // pub fn prefer_extension<'a: 'crit>(mut self, name: impl Into<Cow<'a, CStr>>) -> Self {
    //     self.prefered_extensions.push(name.into());
    //     self
    // }

    pub fn pick_physical_device(
        &self,
        devices: impl Iterator<Item = PhysicalDeviceMetadata>,
    ) -> Option<PhysicalDeviceMetadata> {
        let devices = devices.inspect(|device| tracing::trace!("{:?}", device));

        // Check for min api version
        let devices =
            devices.filter(|device| device.properties.api_version >= self.minimum_api_version);

        // Check if required extensions are present
        let devices = devices.filter(|device| {
            let device_exts = device
                .extensions
                .iter()
                .map(|e| unsafe { CStr::from_ptr(e.extension_name.as_ptr()) })
                .collect::<Vec<_>>();

            for name in &self.required_extensions {
                let found = device_exts
                    .iter()
                    .find(|&&device_ext_name| device_ext_name == name.as_ref())
                    .is_some();

                if !found {
                    return false;
                }
            }

            true
        });

        // Check if the requeste queues are supported
        let devices = devices.filter(|device| {
            self.queue_families.iter().all(|request| {
                request
                    .choose_queue_family_index(&device.queue_families)
                    .is_some()
            })
        });

        // Keep only the most preferred types
        let mut devices = devices
            .fold(Vec::new(), |mut acc, device| {
                let device_type = self
                    .device_type_preference
                    .binary_search(&device.properties.device_type)
                    .unwrap_or(usize::MAX);

                if acc.is_empty() {
                    acc.push((device_type, device));
                    acc
                } else {
                    let preferred_type = acc[0].0;

                    // The preferred type is the lowest value

                    if preferred_type < device_type {
                        acc
                    } else if preferred_type == device_type {
                        acc.push((device_type, device));
                        acc
                    } else {
                        vec![(device_type, device)]
                    }
                }
            })
            .into_iter()
            .map(|(_, d)| d);

        debug_assert!(devices.len() > 0);

        if devices.len() > 1 {
            tracing::info!("Muliple equally suitable devices found, taking the first one");
        }

        devices.next()
    }
}
