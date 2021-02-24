#[derive(Clone, Debug)]
pub enum Errors {
    Base64DecodeError(base64::DecodeError),
    DocxBuildError,
    ParseError,
}
