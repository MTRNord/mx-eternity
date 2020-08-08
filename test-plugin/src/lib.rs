#![no_main]

// Define the functions from the framework which allow access to sending messages
extern "C" {
    fn send_message(
        content_ptr: *const u8,
        content_len: usize,
        room_id_ptr: *const u8,
        room_id_len: usize,
    );
}

// Export a function named "main_plugin". This can be called
// from the framework!
#[no_mangle]
pub extern "C" fn main_plugin() {
    println!("called main_plugin");
    // Call the function we just imported and pass in
    // the offset of our string and its length as parameters.
    let content =
        "{\"type\": \"m.room.message\",\"content\": {\"msgtype\": \"m.text\",\"body\": \"test\"}}";
    let room_id = "!KwXDovBFhYakswlOwN:nordgedanken.dev";
    unsafe {
        send_message(
            content.as_ptr(),
            content.len(),
            room_id.as_ptr(),
            room_id.len(),
        );
    }
}
