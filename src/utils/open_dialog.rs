#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
pub mod open_dialog {

    use dialoguer::theme::ColorfulTheme;
    use error_stack::{IntoReport, ResultExt};

    use dialoguer::Select;

    use crate::dialogs::dialogs::{DialogsErrors, ResultDialogsErr};

    pub fn open_dialog(
        selections: &Vec<String>,
        prompt: &str,
        max_length: Option<u8>,
    ) -> ResultDialogsErr<usize> {
        let max_length = max_length.unwrap_or(255);
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt(prompt)
            .default(0)
            .max_length(max_length.into())
            .items(&selections[..])
            .interact()
            .report()
            .change_context(DialogsErrors::OpenDialogError)?;
        Ok(selection)
    }
}
