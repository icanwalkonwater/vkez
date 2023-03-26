use std::ffi::CStr;

use ash::vk;
use tracing::Level;

pub unsafe extern "system" fn vkez_debug_utils_messenger(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _: *mut std::ffi::c_void,
) -> vk::Bool32 {
    let enabled = match message_severity {
        vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => tracing::event_enabled!(Level::TRACE),
        vk::DebugUtilsMessageSeverityFlagsEXT::INFO => tracing::event_enabled!(Level::INFO),
        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => tracing::event_enabled!(Level::WARN),
        vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => tracing::event_enabled!(Level::ERROR),
        _ => panic!("Unknown message severity"),
    };

    if !enabled {
        return vk::FALSE;
    }

    let p_callback_data = &(*p_callback_data);

    let message_id_name = CStr::from_ptr(p_callback_data.p_message_id_name)
        .to_string_lossy()
        .into_owned();
    let message_id = p_callback_data.message_id_number;
    let message = CStr::from_ptr(p_callback_data.p_message)
        .to_string_lossy()
        .into_owned();

    let queue_labels = if !p_callback_data.p_queue_labels.is_null() {
        std::slice::from_raw_parts(
            p_callback_data.p_queue_labels,
            p_callback_data.queue_label_count as _,
        )
        .into_iter()
        .map(|queue_label| {
            CStr::from_ptr(queue_label.p_label_name)
                .to_string_lossy()
                .into_owned()
        })
        .collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    let command_buffer_labels = if !p_callback_data.p_cmd_buf_labels.is_null() {
        std::slice::from_raw_parts(
            p_callback_data.p_cmd_buf_labels,
            p_callback_data.cmd_buf_label_count as _,
        )
        .into_iter()
        .map(|command_buffer_label| {
            CStr::from_ptr(command_buffer_label.p_label_name)
                .to_string_lossy()
                .into_owned()
        })
        .collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    let objects = if !p_callback_data.p_objects.is_null() {
        std::slice::from_raw_parts(p_callback_data.p_objects, p_callback_data.object_count as _)
            .into_iter()
            .map(|object| {
                let ty = object.object_type;
                let handle = object.object_handle;
                if object.p_object_name.is_null() {
                    format!("({ty:?}) {handle:#x}")
                } else {
                    let name = CStr::from_ptr(object.p_object_name).to_string_lossy();
                    format!("({ty:?}) {name} = {handle:#x}")
                }
            })
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    let message = format!("[{message_type:?}] ({message_id}) {message}");

    macro_rules! event_with_level {
        ($level:expr) => {
            ::tracing::event!(
                $level,
                message,
                ty = ?message_type,
                message_id,
                message_id_name,
                ?queue_labels,
                ?command_buffer_labels,
                ?objects,
            )
        };
    }

    match message_severity {
        vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => event_with_level!(Level::TRACE),
        vk::DebugUtilsMessageSeverityFlagsEXT::INFO => event_with_level!(Level::INFO),
        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => event_with_level!(Level::WARN),
        vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => event_with_level!(Level::ERROR),
        _ => panic!("Unknown message severity"),
    }

    vk::FALSE
}
