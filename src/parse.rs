pub type Result<'a, O> = nom::IResult<&'a [u8], O, Error<'a>>;
pub type Error<'a> = (&'a [u8], nom::error::ErrorKind);
