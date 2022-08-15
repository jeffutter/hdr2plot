use nom::{
    branch::alt,
    bytes::complete::{tag, take_until1},
    character::complete::{digit1, line_ending, multispace0},
    combinator::{map_res, opt, recognize},
    error::ParseError,
    multi::{many0, many1, separated_list1},
    sequence::{delimited, preceded, separated_pair, tuple},
    IResult,
};

use crate::hgrm::{HGRMs, OnePercentile, Percentile, HGRM};

fn decimal_float(s: &str) -> IResult<&str, f64> {
    map_res(recognize(tuple((digit1, tag("."), digit1))), |s: &str| {
        s.parse::<f64>()
    })(s)
}

fn digit_u64(s: &str) -> IResult<&str, u64> {
    map_res(digit1, |s: &str| s.parse::<u64>())(s)
}

fn one_percentile(s: &str) -> IResult<&str, OnePercentile> {
    map_res(
        alt((recognize(tuple((digit1, tag("."), digit1))), tag("inf"))),
        |s: &str| match s {
            "inf" => Ok(OnePercentile::Inf),
            decimal_str => decimal_str.parse::<f64>().map(OnePercentile::Value),
        },
    )(s)
}

fn percentile_line(s: &str) -> IResult<&str, Percentile> {
    let (rest, (value, percentile, total_count, one_percentile)) = tuple((
        ws(decimal_float),
        ws(decimal_float),
        ws(digit_u64),
        one_percentile,
    ))(s)?;

    let percentile = Percentile::new(value, percentile, total_count, one_percentile);
    Ok((rest, percentile))
}

fn ws<'a, F: 'a, O, E: ParseError<&'a str>>(
    inner: F,
) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
    F: Fn(&'a str) -> IResult<&'a str, O, E>,
{
    delimited(multispace0, inner, multispace0)
}

fn aggregate<'a, F1: 'a, F2: 'a, F3: 'a, F4: 'a, G1, G2, O1, O2, E: ParseError<&'a str> + 'a>(
    left_title: F1,
    left: F2,
    right_title: F3,
    right: F4,
) -> impl FnMut(&'a str) -> IResult<&'a str, (O1, O2), E>
where
    F1: Fn(&'a str) -> IResult<&'a str, G1, E>,
    F2: Fn(&'a str) -> IResult<&'a str, O1, E>,
    F3: Fn(&'a str) -> IResult<&'a str, G2, E>,
    F4: Fn(&'a str) -> IResult<&'a str, O2, E>,
{
    delimited(
        tag("#["),
        separated_pair(
            preceded(tuple((left_title, ws(tag("=")))), left),
            ws(tag(",")),
            preceded(tuple((right_title, ws(tag("=")))), right),
        ),
        tag("]"),
    )
}

fn parse_name<'a>(s: &'a str) -> IResult<&str, Option<&'a str>> {
    let (rest, res1) = opt(preceded(multispace0, many1(tag("="))))(s)?;
    match res1 {
        None => Ok((s, None)),
        Some(_) => {
            let (rest, name) = opt(take_until1("="))(rest)?;
            match name {
                None => Ok((s, None)),
                Some(name) => {
                    let (rest, res2) = opt(many1(tag("=")))(rest)?;
                    match res2 {
                        None => Ok((s, None)),
                        Some(_) => Ok((rest, Some(name.trim()))),
                    }
                }
            }
        }
    }
}

fn parse_hgrm(s: &str) -> IResult<&str, HGRM> {
    let (rest, name) = parse_name(s)?;

    let (rest, _) = tuple((
        ws(tag("Value")),
        ws(tag("Percentile")),
        ws(tag("TotalCount")),
        tag("1/(1-Percentile)"),
    ))(rest)?;
    let (rest, _) = many0(line_ending)(rest)?;

    let (rest, percentiles) = separated_list1(line_ending, percentile_line)(rest)?;

    let (rest, _) = many1(line_ending)(rest)?;

    let (rest, (mean, std_deviation)) = aggregate(
        tag("Mean"),
        decimal_float,
        tag("StdDeviation"),
        decimal_float,
    )(rest)?;

    let (rest, _) = many1(line_ending)(rest)?;

    let (rest, (max, total_count)) =
        aggregate(tag("Max"), decimal_float, tag("Total count"), digit_u64)(rest)?;

    let (rest, _) = many1(line_ending)(rest)?;

    let (rest, (buckets, sub_buckets)) =
        aggregate(tag("Buckets"), digit_u64, tag("SubBuckets"), digit_u64)(rest)?;

    let hgrm = HGRM::new()
        .set_name(name)
        .set_mean(mean)
        .set_std_deviation(std_deviation)
        .set_max(max)
        .set_total_count(total_count)
        .set_buckets(buckets)
        .set_sub_buckets(sub_buckets)
        .set_percentiles(percentiles);

    Ok((rest, hgrm))
}

pub fn parse(s: &str) -> IResult<&str, HGRMs> {
    let (rest, hgrms) = many1(parse_hgrm)(s)?;

    Ok((rest, HGRMs::new(hgrms)))
}

#[cfg(test)]
mod tests {
    use crate::hgrm::OnePercentile;

    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[cfg(test)]
    use pretty_assertions::assert_eq;

    #[test]
    fn test_parse() {
        let data = r#"
       Value   Percentile   TotalCount 1/(1-Percentile)

       0.189     0.000000            1         1.00
       2.845     0.100000       348248         1.11
       3.603     0.200000       695644         1.25
       4.135     0.300000      1043796         1.43
       4.551     0.400000      1391678         1.67
       4.919     0.500000      1740028         2.00
       5.095     0.550000      1914665         2.22
      64.767     1.000000      3477000          inf
#[Mean    =        4.881, StdDeviation   =        1.777]
#[Max     =       64.736, Total count    =      3477000]
#[Buckets =           27, SubBuckets     =         2048]
"#;

        let (_, parsed) = parse(data).unwrap();

        let expected = HGRM::new()
            .set_mean(4.881)
            .set_std_deviation(1.777)
            .set_max(64.736)
            .set_total_count(3477000)
            .set_buckets(27)
            .set_sub_buckets(2048)
            .add_percentile(0.189, 0.000000, 1, OnePercentile::Value(1.00))
            .add_percentile(2.845, 0.100000, 348248, OnePercentile::Value(1.11))
            .add_percentile(3.603, 0.200000, 695644, OnePercentile::Value(1.25))
            .add_percentile(4.135, 0.300000, 1043796, OnePercentile::Value(1.43))
            .add_percentile(4.551, 0.400000, 1391678, OnePercentile::Value(1.67))
            .add_percentile(4.919, 0.500000, 1740028, OnePercentile::Value(2.00))
            .add_percentile(5.095, 0.550000, 1914665, OnePercentile::Value(2.22))
            .add_percentile(64.767, 1.000000, 3477000, OnePercentile::Inf);

        assert_eq!(parsed, HGRMs::new(vec![expected]))
    }

    #[test]
    fn test_named() {
        let data = r#"
=== Name 1 ===
       Value   Percentile   TotalCount 1/(1-Percentile)

       0.189     0.000000            1         1.00
       2.845     0.100000       348248         1.11
       3.603     0.200000       695644         1.25
       4.135     0.300000      1043796         1.43
       4.551     0.400000      1391678         1.67
       4.919     0.500000      1740028         2.00
       5.095     0.550000      1914665         2.22
      64.767     1.000000      3477000          inf
#[Mean    =        4.881, StdDeviation   =        1.777]
#[Max     =       64.736, Total count    =      3477000]
#[Buckets =           27, SubBuckets     =         2048]
"#;

        let (_, parsed) = parse(data).unwrap();

        let expected = HGRM::new()
            .set_name(Some("Name 1"))
            .set_mean(4.881)
            .set_std_deviation(1.777)
            .set_max(64.736)
            .set_total_count(3477000)
            .set_buckets(27)
            .set_sub_buckets(2048)
            .add_percentile(0.189, 0.000000, 1, OnePercentile::Value(1.00))
            .add_percentile(2.845, 0.100000, 348248, OnePercentile::Value(1.11))
            .add_percentile(3.603, 0.200000, 695644, OnePercentile::Value(1.25))
            .add_percentile(4.135, 0.300000, 1043796, OnePercentile::Value(1.43))
            .add_percentile(4.551, 0.400000, 1391678, OnePercentile::Value(1.67))
            .add_percentile(4.919, 0.500000, 1740028, OnePercentile::Value(2.00))
            .add_percentile(5.095, 0.550000, 1914665, OnePercentile::Value(2.22))
            .add_percentile(64.767, 1.000000, 3477000, OnePercentile::Inf);

        assert_eq!(parsed, HGRMs::new(vec![expected]))
    }
}
