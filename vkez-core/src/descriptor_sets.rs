use std::{borrow::Cow, slice::from_ref};

use ash::{prelude::VkResult, vk};

fn pool_sizes_for_n<F: RawDescriptorSetInfo + ?Sized>(
    max_sets: u32,
) -> Vec<vk::DescriptorPoolSize> {
    F::POOL_SIZES_FOR_ONE
        .iter()
        .cloned()
        .map(|mut s| {
            s.descriptor_count *= max_sets;
            s
        })
        .collect()
}

pub unsafe trait RawDescriptorSetInfo {
    const LAYOUT_BINDINGS_CREATE_INFO: &'static [vk::DescriptorSetLayoutBinding];
    const LAYOUT_CREATE_INFO: vk::DescriptorSetLayoutCreateInfo;

    const POOL_SIZES_FOR_ONE: &'static [vk::DescriptorPoolSize];

    #[inline]
    unsafe fn create_layout(device: &ash::Device) -> VkResult<vk::DescriptorSetLayout> {
        device.create_descriptor_set_layout(&Self::LAYOUT_CREATE_INFO, None)
    }

    unsafe fn create_pool_for_set(
        device: &ash::Device, max_sets: u32,
    ) -> VkResult<vk::DescriptorPool> {
        let max_sets = max_sets.max(1);
        let sizes = if max_sets == 1 {
            Cow::Borrowed(Self::POOL_SIZES_FOR_ONE)
        } else {
            Cow::Owned(pool_sizes_for_n::<Self>(max_sets))
        };

        device.create_descriptor_pool(
            &vk::DescriptorPoolCreateInfo::builder()
                .max_sets(max_sets)
                .pool_sizes(sizes.as_ref()),
            None,
        )
    }

    unsafe fn allocate_one_set(
        device: &ash::Device, pool: vk::DescriptorPool, layout: vk::DescriptorSetLayout,
    ) -> VkResult<vk::DescriptorSet> {
        Ok(device.allocate_descriptor_sets(
            &vk::DescriptorSetAllocateInfo::builder()
                .descriptor_pool(pool)
                .set_layouts(from_ref(&layout)),
        )?[0])
    }
}
