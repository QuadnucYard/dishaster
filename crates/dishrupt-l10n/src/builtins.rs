use fluent_bundle::{FluentArgs, FluentValue, types::FluentNumber};

pub fn number<'a>(positional: &[FluentValue<'a>], named: &FluentArgs) -> FluentValue<'a> {
    let Some(FluentValue::Number(num)) = positional.first() else {
        return FluentValue::Error;
    };

    let mut num = num.clone();
    num.options.merge(named);
    let options = merge_num_options(&mut num, named);

    format_number(num.value, num.options.maximum_fraction_digits, options.sign)
}

pub fn percent<'a>(positional: &[FluentValue<'a>], named: &FluentArgs) -> FluentValue<'a> {
    let Some(FluentValue::Number(num)) = positional.first() else {
        return FluentValue::Error;
    };

    let mut num = num.clone();
    num.value *= 100.0;
    num.options.merge(named);
    let options = merge_num_options(&mut num, named);

    let formatted = format_number_raw(num.value, num.options.maximum_fraction_digits, options.sign);
    FluentValue::String(format!("{}%", formatted).into())
}

fn format_number<'a>(
    value: f64,
    max_fraction_digits: Option<usize>,
    sign: SignDisplay,
) -> FluentValue<'a> {
    FluentValue::String(format_number_raw(value, max_fraction_digits, sign).into())
}

fn format_number_raw(value: f64, max_fraction_digits: Option<usize>, sign: SignDisplay) -> String {
    let formatted = if let Some(d) = max_fraction_digits {
        format!("{:.*}", d, value.abs())
    } else {
        value.abs().to_string()
    };

    match sign {
        SignDisplay::Always => {
            if value >= 0.0 {
                format!("+{}", formatted)
            } else {
                format!("-{}", formatted)
            }
        }
        SignDisplay::Never => formatted,
        SignDisplay::Auto => {
            if value < 0.0 {
                format!("-{}", formatted)
            } else {
                formatted
            }
        }
    }
}

struct NumFormatOptions {
    sign: SignDisplay,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum SignDisplay {
    Auto,
    Always,
    Never,
}

fn merge_num_options(num: &mut FluentNumber, options: &FluentArgs) -> NumFormatOptions {
    let mut format_options = NumFormatOptions {
        sign: SignDisplay::Auto,
    };

    for (key, value) in options.iter() {
        match (key, value) {
            ("maxfd", FluentValue::Number(n)) => {
                num.options.maximum_fraction_digits = Some(n.into());
            }
            ("minfd", FluentValue::Number(n)) => {
                num.options.minimum_fraction_digits = Some(n.into());
            }
            ("sign", FluentValue::String(s)) => {
                format_options.sign = match s.as_ref() {
                    "always" => SignDisplay::Always,
                    "never" => SignDisplay::Never,
                    _ => SignDisplay::Auto,
                };
            }
            _ => {}
        }
    }

    format_options
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_number(value: f64) -> FluentValue<'static> {
        FluentValue::Number(FluentNumber::new(value, Default::default()))
    }

    fn extract_string<'a>(value: &'a FluentValue) -> &'a str {
        match value {
            FluentValue::String(s) => s.as_ref(),
            _ => panic!("Expected String value"),
        }
    }

    #[test]
    fn test_number_basic() {
        let result = number(&[make_number(42.0)], &FluentArgs::new());
        // With no maxfd specified, default formatting applies
        assert_eq!(extract_string(&result), "42");
    }

    #[test]
    fn test_number_with_max_fraction_digits() {
        let mut args = FluentArgs::new();
        args.set(
            "maxfd",
            FluentValue::Number(FluentNumber::new(2.0, Default::default())),
        );
        #[allow(clippy::approx_constant)]
        let result = number(&[make_number(3.14159)], &args);
        assert_eq!(extract_string(&result), "3.14");
    }

    #[test]
    fn test_number_with_sign_always() {
        let mut args = FluentArgs::new();
        args.set(
            "maxfd",
            FluentValue::Number(FluentNumber::new(1.0, Default::default())),
        );
        args.set("sign", FluentValue::String("always".into()));

        let positive = number(&[make_number(5.5)], &args);
        assert_eq!(extract_string(&positive), "+5.5");

        let negative = number(&[make_number(-5.5)], &args);
        assert_eq!(extract_string(&negative), "-5.5");

        let zero = number(&[make_number(0.0)], &args);
        assert_eq!(extract_string(&zero), "+0.0");
    }

    #[test]
    fn test_number_with_sign_never() {
        let mut args = FluentArgs::new();
        args.set(
            "maxfd",
            FluentValue::Number(FluentNumber::new(1.0, Default::default())),
        );
        args.set("sign", FluentValue::String("never".into()));

        let positive = number(&[make_number(5.5)], &args);
        assert_eq!(extract_string(&positive), "5.5");

        let negative = number(&[make_number(-5.5)], &args);
        assert_eq!(extract_string(&negative), "5.5");
    }

    #[test]
    fn test_number_with_sign_auto() {
        let mut args = FluentArgs::new();
        args.set(
            "maxfd",
            FluentValue::Number(FluentNumber::new(1.0, Default::default())),
        );

        let positive = number(&[make_number(5.5)], &args);
        assert_eq!(extract_string(&positive), "5.5");

        let negative = number(&[make_number(-5.5)], &args);
        assert_eq!(extract_string(&negative), "-5.5");
    }

    #[test]
    fn test_percent_basic() {
        let mut args = FluentArgs::new();
        args.set(
            "maxfd",
            FluentValue::Number(FluentNumber::new(0.0, Default::default())),
        );
        let result = percent(&[make_number(0.5)], &args);
        assert_eq!(extract_string(&result), "50%");
    }

    #[test]
    fn test_percent_with_decimals() {
        let mut args = FluentArgs::new();
        args.set(
            "maxfd",
            FluentValue::Number(FluentNumber::new(1.0, Default::default())),
        );
        let result = percent(&[make_number(0.123)], &args);
        assert_eq!(extract_string(&result), "12.3%");
    }

    #[test]
    fn test_percent_with_sign_always() {
        let mut args = FluentArgs::new();
        args.set(
            "maxfd",
            FluentValue::Number(FluentNumber::new(0.0, Default::default())),
        );
        args.set("sign", FluentValue::String("always".into()));

        let positive = percent(&[make_number(0.15)], &args);
        assert_eq!(extract_string(&positive), "+15%");

        let negative = percent(&[make_number(-0.15)], &args);
        assert_eq!(extract_string(&negative), "-15%");
    }

    #[test]
    fn test_percent_negative() {
        let mut args = FluentArgs::new();
        args.set(
            "maxfd",
            FluentValue::Number(FluentNumber::new(1.0, Default::default())),
        );
        let result = percent(&[make_number(-0.25)], &args);
        assert_eq!(extract_string(&result), "-25.0%");
    }

    #[test]
    fn test_percent_with_sign_never() {
        let mut args = FluentArgs::new();
        args.set(
            "maxfd",
            FluentValue::Number(FluentNumber::new(0.0, Default::default())),
        );
        args.set("sign", FluentValue::String("never".into()));

        let positive = percent(&[make_number(0.2)], &args);
        assert_eq!(extract_string(&positive), "20%");

        let negative = percent(&[make_number(-0.2)], &args);
        assert_eq!(extract_string(&negative), "20%");
    }

    #[test]
    fn test_percent_with_sign_auto() {
        let mut args = FluentArgs::new();
        args.set(
            "maxfd",
            FluentValue::Number(FluentNumber::new(0.0, Default::default())),
        );

        let positive = percent(&[make_number(0.3)], &args);
        assert_eq!(extract_string(&positive), "30%");

        let negative = percent(&[make_number(-0.3)], &args);
        assert_eq!(extract_string(&negative), "-30%");
    }

    #[test]
    fn test_number_error_handling() {
        // Test with no arguments
        let result = number(&[], &FluentArgs::new());
        assert!(matches!(result, FluentValue::Error));

        // Test with non-number argument
        let result = number(
            &[FluentValue::String("not a number".into())],
            &FluentArgs::new(),
        );
        assert!(matches!(result, FluentValue::Error));
    }

    #[test]
    fn test_percent_error_handling() {
        // Test with no arguments
        let result = percent(&[], &FluentArgs::new());
        assert!(matches!(result, FluentValue::Error));

        // Test with non-number argument
        let result = percent(
            &[FluentValue::String("not a number".into())],
            &FluentArgs::new(),
        );
        assert!(matches!(result, FluentValue::Error));
    }
}
