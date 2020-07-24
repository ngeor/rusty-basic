#[cfg(test)]
mod tests {
    use crate::assert_prints;

    #[test]
    fn test_while_wend() {
        let input = "
        A = 1
        WHILE A < 5
            PRINT A
            A = A + 1
        WEND
        ";
        assert_prints!(input, "1", "2", "3", "4");
    }
}
