use num_format::{Locale, ToFormattedString};

pub fn amount(amount: f64) -> String {
    if amount > 1_000. {
        as_integer(amount)
    } else if amount > 0. {
        number_with_precision(amount, 4)
    } else {
        "-".into()
    }
}

pub fn value(value: f64) -> String {
    if value > 0. {
        number_with_decimals(value, 2)
    } else {
        "-".into()
    }
}

fn as_integer(number: f64) -> String {
    let integer = number.floor() as u128;
    integer.to_formatted_string(&Locale::en)
}

fn number_with_decimals(number: f64, decimals: usize) -> String {
    let integer = as_integer(number);
    if decimals == 0 {
        return integer;
    }
    let full = format!("{0:.1$}", number, decimals);
    let (_, decimal) = full.split_once('.').unwrap();
    format!("{integer}.{decimal}")
}

// https://stackoverflow.com/questions/60497397/how-do-you-format-a-float-to-the-first-significant-decimal-and-with-specified-pr
fn number_with_precision(number: f64, precision: usize) -> String {
    let decimals = if number >= 1. {
        let n = number.log10().ceil() as usize;
        if n <= precision {
            precision - n
        } else {
            0
        }
    } else if number > 0. {
        let n = number.log10().ceil().abs() as usize;
        precision + n
    } else {
        0
    };
    number_with_decimals(number, decimals)
}
