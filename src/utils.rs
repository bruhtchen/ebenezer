// ------------------------------------------------------------
// UTILS
// ------------------------------------------------------------
/// Parse a string (hopefully numerical) into an amount in cents.
pub fn parse_into_cents(value: &str) -> i64 {
    let splitted: Vec<&str> = value.split(&['.', ','][..]).collect();

    if splitted.len() > 2 {
        panic!("Amounts shouldn't have more than one decimal separator. ({})", value);
    }

    let main:i64 = splitted[0].parse()
        .expect("Input isn't a valid amount !");

    let str_cents = splitted.get(1).or(Some(&"0"))
        .expect("Default value 0 should always be present.");

    if str_cents.len() > 2 {
        panic!("More than two digits after decimal place. ({})", value)
    }

    let padded_cents = format!("{:0<2}", str_cents);

    let cents: i64 = padded_cents.parse().expect("Input isn't a valid amount !");
    return main * 100 + cents;
}

pub fn print_in_currency(amount: i64) -> String {
    let currency = crate::get_currency();
    let cents = amount % 100;
    let money = amount / 100;
    return format!("{},{:02}{}", money, cents, currency);
}
