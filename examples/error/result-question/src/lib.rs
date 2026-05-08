/// Result + `?` operator desugar through Try / FromResidual.
pub fn result_question() {
    fn parse_pair(s: &str) -> Result<(i32, i32), std::num::ParseIntError> {
        let mut it = s.split(',');
        let a: i32 = it.next().unwrap_or("0").parse()?;
        let b: i32 = it.next().unwrap_or("0").parse()?;
        Ok((a, b))
    }
    let _ = parse_pair("1,2");
    let _ = parse_pair("oops");
}
