use std::{ffi::CStr, mem, rc::Rc, slice::from_ref};

use ash::{util::Align, vk};
use tracing::Level;
use vk_mem::Alloc;
use vkez::ash;
use vkez::bootstrap::{AshDeviceExt, AshInstanceExt, PhysicalDeviceCriteria, QueueFamilyRequest};
use vkez::tracing;

fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .compact()
        .init();

    let entry = ash::Entry::linked();

    let (instance, debug_stuff) = unsafe {
        ash::Instance::builder()
            .api_version(vk::API_VERSION_1_1)
            .app_name("vkez demo")
            .engine_name("vkez")
            .enable_default_debug_utils()
            .create_instance(&entry)?
    };

    let (debug_utils, debug_messenger) = debug_stuff.unwrap();

    let compute_queue = QueueFamilyRequest::empty()
        .require_compute()
        .prefer_alone()
        .amount(1);

    let (device, device_metadata) = unsafe {
        ash::Device::builder()
            .physical_device_criteria(
                PhysicalDeviceCriteria::empty()
                    .prefer_device_type(vk::PhysicalDeviceType::DISCRETE_GPU)
                    .request_queue_family(&compute_queue),
            )
            .create_device(&instance)?
    };

    let device_name = unsafe {
        CStr::from_ptr(
            device_metadata
                .physical_device
                .properties
                .device_name
                .as_ptr(),
        )
    };
    tracing::info!("Using physical device {:?}", device_name);

    let compute_queue = unsafe { device_metadata.get_device_queue(&device, compute_queue, 0)? };

    let allocator = vk_mem::Allocator::new(
        vk_mem::AllocatorCreateInfo::new(
            Rc::new(&instance),
            Rc::new(&device),
            device_metadata.physical_device.handle,
        )
        .vulkan_api_version(vk::API_VERSION_1_1),
    )?;

    let mut buffer_a = unsafe {
        allocator.create_buffer(
            &vk::BufferCreateInfo::builder()
                .size(256 * mem::size_of::<f32>() as vk::DeviceSize)
                .usage(vk::BufferUsageFlags::STORAGE_BUFFER)
                .sharing_mode(vk::SharingMode::EXCLUSIVE),
            &vk_mem::AllocationCreateInfo {
                usage: vk_mem::MemoryUsage::AutoPreferDevice,
                flags: vk_mem::AllocationCreateFlags::HOST_ACCESS_SEQUENTIAL_WRITE,
                ..Default::default()
            },
        )?
    };

    let mut buffer_b = unsafe {
        allocator.create_buffer(
            &vk::BufferCreateInfo::builder()
                .size(256 * mem::size_of::<f32>() as vk::DeviceSize)
                .usage(vk::BufferUsageFlags::STORAGE_BUFFER)
                .sharing_mode(vk::SharingMode::EXCLUSIVE),
            &vk_mem::AllocationCreateInfo {
                usage: vk_mem::MemoryUsage::AutoPreferDevice,
                flags: vk_mem::AllocationCreateFlags::HOST_ACCESS_SEQUENTIAL_WRITE,
                ..Default::default()
            },
        )?
    };

    let mut buffer_c = unsafe {
        allocator.create_buffer(
            &vk::BufferCreateInfo::builder()
                .size(256 * mem::size_of::<f32>() as vk::DeviceSize)
                .usage(vk::BufferUsageFlags::STORAGE_BUFFER)
                .sharing_mode(vk::SharingMode::EXCLUSIVE),
            &vk_mem::AllocationCreateInfo {
                usage: vk_mem::MemoryUsage::AutoPreferDevice,
                flags: vk_mem::AllocationCreateFlags::HOST_ACCESS_SEQUENTIAL_WRITE,
                ..Default::default()
            },
        )?
    };

    unsafe {
        let a_info = allocator.get_allocation_info(&buffer_a.1)?;
        let a_ptr = allocator.map_memory(&mut buffer_a.1)?;
        let mut align = Align::<f32>::new(a_ptr as _, mem::align_of::<f32>() as _, a_info.size);

        let data = [1.0; 256];
        align.copy_from_slice(&data);
        allocator.flush_allocation(&buffer_a.1, 0, vk::WHOLE_SIZE as _)?;
        allocator.unmap_memory(&mut buffer_a.1);

        let b_info = allocator.get_allocation_info(&buffer_b.1)?;
        let b_ptr = allocator.map_memory(&mut buffer_b.1)?;
        let mut align = Align::<f32>::new(b_ptr as _, mem::align_of::<f32>() as _, b_info.size);

        let data = [2.0; 256];
        align.copy_from_slice(&data);
        allocator.flush_allocation(&buffer_b.1, 0, vk::WHOLE_SIZE as _)?;
        allocator.unmap_memory(&mut buffer_b.1);
    }

    let descriptor_pool = unsafe {
        device.create_descriptor_pool(
            &vk::DescriptorPoolCreateInfo::builder()
                .max_sets(1)
                .pool_sizes(from_ref(
                    &vk::DescriptorPoolSize::builder()
                        .ty(vk::DescriptorType::STORAGE_BUFFER)
                        .descriptor_count(3),
                )),
            None,
        )?
    };

    let descriptor_set_layout = unsafe {
        device.create_descriptor_set_layout(
            &vk::DescriptorSetLayoutCreateInfo::builder().bindings(&[
                vk::DescriptorSetLayoutBinding::builder()
                    .binding(0)
                    .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                    .descriptor_count(1)
                    .stage_flags(vk::ShaderStageFlags::COMPUTE)
                    .build(),
                vk::DescriptorSetLayoutBinding::builder()
                    .binding(1)
                    .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                    .descriptor_count(1)
                    .stage_flags(vk::ShaderStageFlags::COMPUTE)
                    .build(),
                vk::DescriptorSetLayoutBinding::builder()
                    .binding(2)
                    .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                    .descriptor_count(1)
                    .stage_flags(vk::ShaderStageFlags::COMPUTE)
                    .build(),
            ]),
            None,
        )?
    };

    let descriptor_set = unsafe {
        device.allocate_descriptor_sets(
            &vk::DescriptorSetAllocateInfo::builder()
                .descriptor_pool(descriptor_pool)
                .set_layouts(from_ref(&descriptor_set_layout)),
        )?[0]
    };

    unsafe {
        device.update_descriptor_sets(
            &[
                vk::WriteDescriptorSet::builder()
                    .dst_set(descriptor_set)
                    .dst_binding(0)
                    .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                    .buffer_info(from_ref(
                        &vk::DescriptorBufferInfo::builder()
                            .buffer(buffer_a.0)
                            .offset(0)
                            .range(vk::WHOLE_SIZE),
                    ))
                    .build(),
                vk::WriteDescriptorSet::builder()
                    .dst_set(descriptor_set)
                    .dst_binding(1)
                    .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                    .buffer_info(from_ref(
                        &vk::DescriptorBufferInfo::builder()
                            .buffer(buffer_b.0)
                            .offset(0)
                            .range(vk::WHOLE_SIZE),
                    ))
                    .build(),
                vk::WriteDescriptorSet::builder()
                    .dst_set(descriptor_set)
                    .dst_binding(2)
                    .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                    .buffer_info(from_ref(
                        &vk::DescriptorBufferInfo::builder()
                            .buffer(buffer_c.0)
                            .offset(0)
                            .range(vk::WHOLE_SIZE),
                    ))
                    .build(),
            ],
            &[],
        );
    }

    pub mod desc_set {
        use ash::vk::TaggedStructure;
        use vkez::ash;

        const LAYOUT_BINDINGS_CREATE_INFO: [ash::vk::DescriptorSetLayoutBinding; 0] = [];
        const SET_CREATE_INFO: ash::vk::DescriptorSetLayoutCreateInfo =
            ash::vk::DescriptorSetLayoutCreateInfo {
                s_type: ash::vk::DescriptorSetLayoutCreateInfo::STRUCTURE_TYPE,
                p_next: std::ptr::null(),
                flags: ash::vk::DescriptorSetLayoutCreateFlags::empty(),
                binding_count: LAYOUT_BINDINGS_CREATE_INFO.len() as _,
                p_bindings: LAYOUT_BINDINGS_CREATE_INFO.as_ptr(),
            };
    }

    // #[vkez_macros::shader_set]
    // mod shader_set {
    //     use ash::vk;

    //     unsafe trait Shader {
    //         const CODE: &'static [u32];
    //         const ENTRY_POINT: &'static [u8];
    //         const STAGES: vk::ShaderStageFlags;

    //         fn shader_module_create_info() -> vk::ShaderModuleCreateInfo {
    //             todo!()
    //         }
    //     }

    //     unsafe trait DescriptorSet {}

    //     struct MyComputeShader;

    //     unsafe impl Shader for MyComputeShader {
    //         const CODE: &'static [u32] = &[1, 2];
    //         const ENTRY_POINT: &'static [u8] = b"main\0";
    //         const STAGES: vk::ShaderStageFlags = vk::ShaderStageFlags::COMPUTE;
    //     }

    //     trait ComputeShader: Shader {}
    //     impl ComputeShader for MyComputeShader {}
    // }

    #[vkez_macros::shader_module("./examples/add.comp.glsl", kind = "Compute")]
    pub mod compute_shader_module {}

    let compute_shader = unsafe {
        device.create_shader_module(
            &vk::ShaderModuleCreateInfo::builder().code(&compute_shader_module::CODE),
            None,
        )?
    };

    let compute_pipeline_layout = unsafe {
        device.create_pipeline_layout(
            &vk::PipelineLayoutCreateInfo::builder().set_layouts(from_ref(&descriptor_set_layout)),
            None,
        )?
    };

    const ENTRY_POINT: &[u8] = b"main\0";

    let compute_pipeline = unsafe {
        device
            .create_compute_pipelines(
                vk::PipelineCache::null(),
                from_ref(
                    &vk::ComputePipelineCreateInfo::builder()
                        .stage(
                            *vk::PipelineShaderStageCreateInfo::builder()
                                .name(CStr::from_ptr(ENTRY_POINT.as_ptr() as _))
                                .stage(vk::ShaderStageFlags::COMPUTE)
                                .module(compute_shader),
                        )
                        .layout(compute_pipeline_layout),
                ),
                None,
            )
            .unwrap()[0]
    };

    unsafe {
        device.destroy_shader_module(compute_shader, None);
    }

    let command_pool = unsafe {
        device.create_command_pool(
            &vk::CommandPoolCreateInfo::builder().queue_family_index(compute_queue.1),
            None,
        )?
    };

    let command_buffer = unsafe {
        device.allocate_command_buffers(
            &vk::CommandBufferAllocateInfo::builder()
                .command_pool(command_pool)
                .command_buffer_count(1)
                .level(vk::CommandBufferLevel::PRIMARY),
        )?[0]
    };

    unsafe {
        device.begin_command_buffer(
            command_buffer,
            &vk::CommandBufferBeginInfo::builder()
                .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT),
        )?;

        device.cmd_bind_descriptor_sets(
            command_buffer,
            vk::PipelineBindPoint::COMPUTE,
            compute_pipeline_layout,
            0,
            &[descriptor_set],
            &[],
        );
        device.cmd_bind_pipeline(
            command_buffer,
            vk::PipelineBindPoint::COMPUTE,
            compute_pipeline,
        );
        device.cmd_dispatch(command_buffer, 1, 1, 1);

        device.end_command_buffer(command_buffer)?;
    }

    let fence = unsafe { device.create_fence(&vk::FenceCreateInfo::builder(), None)? };
    unsafe {
        device.queue_submit(
            compute_queue.0,
            from_ref(&vk::SubmitInfo::builder().command_buffers(&[command_buffer])),
            fence,
        )?;
    }

    unsafe {
        device.wait_for_fences(&[fence], true, u64::MAX)?;

        device.destroy_fence(fence, None);
    }

    unsafe {
        let c_info = allocator.get_allocation_info(&buffer_c.1)?;
        let c_ptr = allocator.map_memory(&mut buffer_c.1)?;
        let mut align = Align::<f32>::new(c_ptr as _, mem::align_of::<f32>() as _, c_info.size);

        let data = align.iter_mut().map(|f| *f).collect::<Vec<_>>();
        println!("{data:?}");
        allocator.unmap_memory(&mut buffer_c.1);
    }

    unsafe {
        device.destroy_command_pool(command_pool, None);
        device.destroy_pipeline(compute_pipeline, None);
        device.destroy_pipeline_layout(compute_pipeline_layout, None);

        device.destroy_descriptor_pool(descriptor_pool, None);
        device.destroy_descriptor_set_layout(descriptor_set_layout, None);

        allocator.destroy_buffer(buffer_c.0, buffer_c.1);
        allocator.destroy_buffer(buffer_b.0, buffer_b.1);
        allocator.destroy_buffer(buffer_a.0, buffer_a.1);

        drop(allocator);

        device.destroy_device(None);
        debug_utils.destroy_debug_utils_messenger(debug_messenger, None);
        instance.destroy_instance(None);
    }

    Ok(())
}
