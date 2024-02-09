use std::ops::Range;

use rug::Float;

pub const DISPLAY_DIGITS: usize = 32;
fn insert_delimeter(str: &str, i: usize) -> String {
    let (l, r) = str.split_at(i);
    //println!("{str}, {l}, {r}, e:{exp}");
    let r = r.trim_end_matches('0');

    match (l, r) {
        (l, "") => l.to_string(),
        (l, r) => format!("{l}.{r}"),
    }
}

pub fn disp_num(num: &Float, digits: usize) -> Option<String> {
    let (sign, str, exp) = num.to_sign_string_exp(10, Some(digits));

    let exp = match exp {
        Some(exp) => exp,
  
        None => {return Some(str)}
    };
    let digits = digits as i32;
    let s = if 0 < exp && exp < digits {
        insert_delimeter(&str, exp as usize)
    }  else if exp == digits {
        str
    } else if -digits < exp && exp <= 0 {
        let zeroes = (exp).abs();
        format!("0.{}{}", "0".repeat(zeroes as usize), str.trim_end_matches('0'))

    } else {
        format!("{}e{}", insert_delimeter(&str, 1), exp - 1)
    };
    Some(format!("{}{s}", if sign { "-" } else { "" }))
}

pub fn get_ident_at_end<'a>(input: &'a str) -> Option<&'a str> {
    get_ident_range(input, input.len()).map(|r| &input[r])
}
pub fn get_ident_range(input: &str, ident_end: usize) -> Option<Range<usize>> {
    let mut last_char_index = None;
    for (i, c) in input[..ident_end].char_indices().rev() {
        if c.is_alphabetic() {
            last_char_index = Some(i);
        }
        if !c.is_alphanumeric() {
            break;
        }
    }
    last_char_index.map(|i| i..ident_end)
}



#[cfg(test)]
mod tests {
    use rug::ops::CompleteRound;

    use super::*;

    fn assert_num(expected: &'static str, num: &'static str, digits: usize) {
        assert_eq!(
            expected,
            disp_num(&Float::parse(num).unwrap().complete(256), digits).unwrap()
        )
    }

    #[test]
    fn disp_num1() {
        assert_num("1234", "1234.11", 4);
        assert_num("-1234", "-1234.11", 4);
        assert_num("12.34", "12.340000", 6);
        assert_num("0.0123", "0.01230", 4);
        assert_num("0.0001234", "0.0001234", 4);
        assert_num("1.2e4", "12300", 2);
        assert_num("0", "000", 2);
        assert_num("0.0123", "0.01233", 3);
        assert_num("1.23e-6", "0.000001233", 3);
        assert_num("0.3", "0.3", 16);
    }

    #[test]
    fn test_get_ident_name() {
        assert_eq!(get_ident_at_end("1abc"), Some("abc"));
        assert_eq!(get_ident_at_end("abc+bob1bob1"), Some("bob1bob1"));
        assert_eq!(get_ident_at_end("abc "), None)
    }
}
