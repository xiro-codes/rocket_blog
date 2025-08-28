use rocket::fairing::AdHoc;
use crate::controllers::SelectionOption;

/// Hello World service for managing selection options
pub struct HelloWorldService {
    selection_options: Vec<SelectionOption>,
}

impl HelloWorldService {
    pub fn new() -> Self {
        Self {
            selection_options: vec![
                SelectionOption {
                    id: 1,
                    name: "Option A".to_string(),
                    description: "This is the first option you can choose".to_string(),
                },
                SelectionOption {
                    id: 2,
                    name: "Option B".to_string(),
                    description: "This is the second option available".to_string(),
                },
                SelectionOption {
                    id: 3,
                    name: "Option C".to_string(),
                    description: "This is the third and final option".to_string(),
                },
            ],
        }
    }

    pub fn get_selection_options(&self) -> Vec<SelectionOption> {
        self.selection_options.clone()
    }

    pub fn get_option_by_id(&self, id: u32) -> Option<&SelectionOption> {
        self.selection_options.iter().find(|opt| opt.id == id)
    }

    /// Create a fairing that initializes the service
    pub fn fairing() -> AdHoc {
        AdHoc::on_ignite("Hello World Service", |rocket| async {
            rocket.manage(HelloWorldService::new())
        })
    }
}