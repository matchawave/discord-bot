use syn::{
    parse::{Parse, ParseStream},
    Ident, Lit, Token,
};

pub struct CacheAttribute {
    key: Ident,
    _eq: Token![=],
    value: Lit,
}

impl CacheAttribute {
    pub fn key(&self) -> &Ident {
        &self.key
    }

    pub fn value(&self) -> &Lit {
        &self.value
    }
}

impl Parse for CacheAttribute {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            key: input.parse()?,
            _eq: input.parse()?,
            value: input.parse()?,
        })
    }
}

pub fn parse_time<T>(input: T) -> Result<u64, String>
where
    T: Into<String>,
{
    let mut current_number = String::new();
    let mut current_unit = String::new();
    let mut is_parsing_number = true;

    let s = input.into();

    for (i, c) in s.chars().enumerate() {
        if i == 0 && !c.is_ascii_digit() {
            return Err(format!("Invalid duration format: `{}`", s));
        }
        if c.is_ascii_digit() && is_parsing_number {
            current_number.push(c);
        } else if c.is_ascii_alphabetic() {
            current_unit.push(c);
            is_parsing_number = false;
        } else {
            return Err(format!("Invalid duration format: `{}`", s));
        }
    }

    if current_number.is_empty() && current_unit.is_empty() {
        return Err(format!("Invalid duration format: `{}`", s));
    }
    let number: u64 = current_number
        .parse()
        .map_err(|_| format!("Invalid number in duration: `{}`", current_number))?;
    current_number.clear();

    let seconds = match current_unit.as_str() {
        "s" | "sec" | "secs" | "second" | "seconds" => number,
        "m" | "min" | "mins" | "minute" | "minutes" => number * 60,
        "h" | "hr" | "hrs" | "hour" | "hours" => number * 3600,
        "d" | "day" | "days" => number * 86400,
        "w" | "week" | "weeks" => number * 604800,
        _ => {
            return Err(format!("Invalid time unit in duration: `{}`", current_unit));
        }
    };
    Ok(seconds)
}
