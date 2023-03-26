use std::borrow::Borrow;

use ash::vk;

use super::{PhysicalDeviceCriteria, PhysicalDeviceMetadata, QueueFamilyRequest};

#[derive(Default)]
pub struct DeviceBuilder<'a> {
    physical_device_criteria: Option<PhysicalDeviceCriteria<'a>>,
}

impl<'builder> DeviceBuilder<'builder> {
    pub fn physical_device_criteria<'a: 'builder>(
        mut self,
        criteria: PhysicalDeviceCriteria<'a>,
    ) -> Self {
        self.physical_device_criteria = Some(criteria);
        self
    }

    pub unsafe fn create_device(
        self,
        instance: impl Borrow<ash::Instance>,
    ) -> ash::prelude::VkResult<(ash::Device, DeviceMetadata)> {
        let instance: &ash::Instance = instance.borrow();

        let physical_devices = instance.enumerate_physical_devices()?;

        let Some(physical_device_criteria) = self.physical_device_criteria.as_ref() else {
            tracing::error!("No physical device criteria provided !");
            return Err(vk::Result::ERROR_UNKNOWN);
        };

        let physical_devices = physical_devices.into_iter().map(|device| {
            let features = instance.get_physical_device_features(device);
            let properties = instance.get_physical_device_properties(device);
            let extensions = instance
                .enumerate_device_extension_properties(device)
                .unwrap();

            let queue_families = instance.get_physical_device_queue_family_properties(device);

            PhysicalDeviceMetadata {
                handle: device,
                features,
                properties,
                extensions,
                queue_families,
            }
        });

        let physical_device = physical_device_criteria.pick_physical_device(physical_devices);

        let Some(physical_device) = physical_device else {
            tracing::error!("No suitable physical device found");
            return Err(vk::Result::ERROR_UNKNOWN);
        };

        // SAFETY: each elements references the priority vec stored in physical_device_criteria[].queue_families.priorities
        let queues = physical_device_criteria
            .queue_families
            .iter()
            .map(|q| {
                q.get_create_info(&physical_device.queue_families)
                    .unwrap()
                    .build()
            })
            .collect::<Vec<_>>();

        let extensions = physical_device_criteria
            .required_extensions
            .iter()
            .map(|e| e.as_ptr())
            .collect::<Vec<_>>();

        instance
            .create_device(
                physical_device.handle,
                &vk::DeviceCreateInfo::builder()
                    .queue_create_infos(&queues)
                    .enabled_extension_names(&extensions),
                None,
            )
            .map(|i| (i, DeviceMetadata { physical_device }))
    }
}

pub trait AshDeviceExt {
    fn builder() -> DeviceBuilder<'static>;
}

impl AshDeviceExt for ash::Device {
    fn builder() -> DeviceBuilder<'static> {
        DeviceBuilder::default()
    }
}

pub struct DeviceMetadata {
    pub physical_device: PhysicalDeviceMetadata,
}

impl DeviceMetadata {
    pub unsafe fn get_device_queue(
        &self,
        device: impl Borrow<ash::Device>,
        request: impl Borrow<QueueFamilyRequest>,
        index: u32,
    ) -> ash::prelude::VkResult<(vk::Queue, u32)> {
        let device: &ash::Device = device.borrow();
        let request: &QueueFamilyRequest = request.borrow();
        let Some(family_index) = request.choose_queue_family_index(&self.physical_device.queue_families) else {
            return Err(vk::Result::ERROR_UNKNOWN);
        };

        Ok((device.get_device_queue(family_index, index), family_index))
    }
}
