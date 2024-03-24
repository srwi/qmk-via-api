use mockall::automock;

pub struct KeyboardApi;

#[automock]
pub trait KeyboardApiMethods {
    fn hid_command(&self, command: u8, data: Vec<u8>) -> Option<Vec<u8>>;

    fn get_protocol_version(&self) -> Option<u8>;
}
impl KeyboardApiMethods for KeyboardApi {
    fn hid_command(&self, command: u8, data: Vec<u8>) -> Option<Vec<u8>> {
        Some(vec![0, 8])
    }

    fn get_protocol_version(&self) -> Option<u8> {
        let response = self.hid_command(1, vec![])?;
        Some(response[1])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;

    #[test]
    fn test_get_protocol_version() {
        let mut mock = MockKeyboardApiMethods::new();

        mock.expect_hid_command().returning(|_, _| Some(vec![0, 8]));
        assert_eq!(mock.get_protocol_version(), Some(8));
    }
}
