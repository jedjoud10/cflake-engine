use ash::vk;
use std::{
    borrow::Cow,
    ffi::{c_void, CStr},
};

// Create the debug utils create info
#[cfg(debug_assertions)]
pub(super) unsafe fn create_debug_messenger_create_info(
) -> vk::DebugUtilsMessengerCreateInfoEXT {
    *vk::DebugUtilsMessengerCreateInfoEXT::builder()
        .message_severity(
            vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
        )
        .message_type(
            vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
                | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
        )
        .pfn_user_callback(Some(debug_callback))
}

// Debug callback that is invoked from the debug messenger
#[cfg(debug_assertions)]
pub(super) unsafe extern "system" fn debug_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _cvoid: *mut c_void,
) -> u32 {
    let callback_data = *p_callback_data;
    let message_id_number: i32 =
        callback_data.message_id_number as i32;

    let message_id_name = if callback_data.p_message_id_name.is_null()
    {
        Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message_id_name)
            .to_string_lossy()
    };

    let message = if callback_data.p_message.is_null() {
        Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message).to_string_lossy()
    };

    pub const VERBOSE: u32 = 0b1;
    pub const INFO: u32 = 0b1_0000;
    pub const WARNING: u32 = 0b1_0000_0000;
    pub const ERROR: u32 = 0b1_0000_0000_0000;

    match message_severity.as_raw() {
        VERBOSE => log::debug!(
            "{:?} [{} ({})] : {}\n",
            message_type,
            message_id_name,
            &message_id_number.to_string(),
            message,
        ),
        INFO => log::debug!(
            "{:?} [{} ({})] : {}\n",
            message_type,
            message_id_name,
            &message_id_number.to_string(),
            message,
        ),
        WARNING => log::warn!(
            "{:?} [{} ({})] : {}\n",
            message_type,
            message_id_name,
            &message_id_number.to_string(),
            message,
        ),
        ERROR => log::error!(
            "{:?} [{} ({})] : {}\n",
            message_type,
            message_id_name,
            &message_id_number.to_string(),
            message,
        ),
        _ => {}
    }

    vk::FALSE
}
