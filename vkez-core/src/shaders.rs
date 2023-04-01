use std::ffi::CStr;

use ash::{prelude::VkResult, vk};

pub unsafe trait RawShaderInfo {
    const VIBE_CHECK: &'static str;
    const CODE: &'static [u32];
    const STAGE: vk::ShaderStageFlags;

    fn entry_point() -> &'static CStr;

    #[inline]
    unsafe fn create_shader_module(device: &ash::Device) -> VkResult<vk::ShaderModule> {
        device.create_shader_module(
            &vk::ShaderModuleCreateInfo::builder().code(Self::CODE),
            None,
        )
    }

    #[inline]
    unsafe fn pipeline_shader_stage_info(
        module: vk::ShaderModule,
    ) -> vk::PipelineShaderStageCreateInfo {
        vk::PipelineShaderStageCreateInfo::builder()
            .stage(Self::STAGE)
            .module(module)
            .name(Self::entry_point())
            .build()
    }
}
