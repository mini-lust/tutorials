#[cfg(test)]
use nom::IResult;

#[cfg(test)]
#[allow(unused)]
pub fn assert_pair_eq<T>(input: IResult<&str, T>, expected: T)
where
    T: PartialEq + std::fmt::Debug,
{
    assert!(input.is_ok());
    assert_eq!(input.unwrap().1, expected);
}

#[cfg(test)]
#[allow(unused)]
pub fn assert_list_eq<'a, T, IR, IT>(input: IR, expected: IT)
where
    IR: IntoIterator<Item = IResult<&'a str, T>>,
    IT: IntoIterator<Item = T>,
    T: PartialEq + std::fmt::Debug,
{
    input
        .into_iter()
        .zip(expected.into_iter())
        .for_each(|(i, e)| assert_pair_eq(i, e))
}

#[cfg(test)]
#[allow(unused)]
pub fn assert_err<T>(input: IResult<&str, T>)
where
    T: PartialEq + std::fmt::Debug,
{
    assert!(input.is_err());
}

#[cfg(test)]
#[allow(unused)]
pub fn assert_list_err<'a, T, IR>(input: IR)
where
    IR: IntoIterator<Item = IResult<&'a str, T>>,
    T: PartialEq + std::fmt::Debug,
{
    input.into_iter().for_each(|i| assert_err(i));
}

#[cfg(test)]
#[allow(unused)]
pub fn assert_list_eq_with_f<'a, T, IS, ES, ISI, ESI, IF, EF>(
    input: IS,
    expected: ES,
    input_f: IF,
    expected_f: EF,
) where
    T: PartialEq + std::fmt::Debug,
    IS: IntoIterator<Item = ISI>,
    ES: IntoIterator<Item = ESI>,
    IF: Fn(ISI) -> IResult<&'a str, T>,
    EF: Fn(ESI) -> T,
{
    input
        .into_iter()
        .zip(expected.into_iter())
        .for_each(|(i, e)| assert_pair_eq(input_f(i), expected_f(e)))
}

#[cfg(test)]
#[allow(unused)]
pub fn assert_list_err_with_f<'a, T, IS, ISI, IF>(input: IS, input_f: IF)
where
    T: PartialEq + std::fmt::Debug,
    IS: IntoIterator<Item = ISI>,
    IF: Fn(ISI) -> IResult<&'a str, T>,
{
    input.into_iter().for_each(|i| assert_err(input_f(i)))
}
