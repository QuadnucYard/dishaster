use fluent::{FluentArgs, FluentValue, types::FluentNumber};

pub fn number<'a>(positional: &[FluentValue<'a>], named: &FluentArgs) -> FluentValue<'a> {
    let Some(FluentValue::Number(num)) = positional.first() else {
        return FluentValue::Error;
    };

    let mut num = num.clone();
    num.options.merge(named);
    merge_num_options(&mut num, named);

    if let Some(d) = num.options.maximum_fraction_digits {
        FluentValue::String(format!("{:.*}", d, num.value,).into())
    } else {
        FluentValue::Number(num)
    }
}

pub fn percent<'a>(positional: &[FluentValue<'a>], named: &FluentArgs) -> FluentValue<'a> {
    let Some(FluentValue::Number(num)) = positional.first() else {
        return FluentValue::Error;
    };

    let mut num = num.clone();
    num.value *= 100.0;
    num.options.merge(named);
    merge_num_options(&mut num, named);

    if let Some(d) = num.options.maximum_fraction_digits {
        FluentValue::String(format!("{:.*}%", d, num.value,).into())
    } else {
        FluentValue::Number(num)
    }
}

fn merge_num_options(num: &mut FluentNumber, options: &FluentArgs) {
    for (key, value) in options.iter() {
        match (key, value) {
            ("maxfd", FluentValue::Number(n)) => {
                num.options.maximum_fraction_digits = Some(n.into());
            }
            ("minfd", FluentValue::Number(n)) => {
                num.options.minimum_fraction_digits = Some(n.into());
            }
            _ => {}
        }
    }
}
