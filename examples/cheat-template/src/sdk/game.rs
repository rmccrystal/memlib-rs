use memlib::memory::*;

/// A struct containing information and methods for the game.
/// This struct will be passed into the main hack loop and used accordingly.
pub struct Game {
    pub base_address: Address
}

impl Game {
    /// Creates a new `Game` struct using a process handle
    pub fn new(handle: Handle) -> Result<Self> {
        // Set the global handle so we can use it anywhere
        set_global_handle(handle);

        // Get the base address or return an error
        let base_address = get_module(crate::PROCESS_NAME)
            .ok_or_else(|| format!("Error getting module {}", crate::PROCESS_NAME))?
            .base_address;

        Ok(Self {
            base_address
        })
    }
}