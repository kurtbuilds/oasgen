pub use openapiv3::*;
pub use oasgen_macro::OaSchema;
pub use oasgen_core as core;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
