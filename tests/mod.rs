#[cfg(test)]
mod tests {
    use insta::assert_yaml_snapshot;
    use rtf_rs::docx::Docx;
    #[test]
    fn rtf_test_1() {
        let bytes = include_bytes!("./mocks/helloworld.rtf");
        let rtf = rtf_rs::Rtf::from_bytes(bytes).expect("must parse");
        assert_yaml_snapshot!(rtf.into_docx().unwrap());
    }
}
