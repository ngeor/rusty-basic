#[cfg(test)]
mod tests {
    use super::super::test_utils::*;

    #[test]
    fn test_while_wend() {
        let input = "
        A = 1
        WHILE A < 5
            PRINT A
            A = A + 1
        WEND
        ";
        assert_eq!(interpret(input).stdlib.output, vec!["1", "2", "3", "4"]);
    }
}
