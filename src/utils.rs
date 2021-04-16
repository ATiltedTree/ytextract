use std::ops::RangeInclusive;

pub trait String {
    fn split_contained<'a>(&'a self, range: RangeInclusive<&str>) -> Option<&'a str>;

    fn split_start_off<'a>(&'a self, delimiter: &str) -> Option<&'a str>;
    fn split_end_off<'a>(&'a self, delimiter: &str) -> Option<&'a str>;
}

impl String for str {
    #[inline]
    fn split_contained<'a>(&'a self, range: RangeInclusive<&str>) -> Option<&'a str> {
        self.split_start_off(range.start())?
            .split_end_off(range.end())
    }

    #[inline]
    fn split_start_off<'a>(&'a self, delimiter: &str) -> Option<&'a str> {
        self.split_once(delimiter).map(|(_, s)| s)
    }

    #[inline]
    fn split_end_off<'a>(&'a self, delimiter: &str) -> Option<&'a str> {
        self.split_once(delimiter).map(|(s, _)| s)
    }
}
