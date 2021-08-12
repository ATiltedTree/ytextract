pub mod browse;
pub mod innertube;
pub mod player_response;
pub mod tv_config;

pub fn parse_subscribers(value: &str) -> Option<u64> {
    let last = value.chars().last()?;
    if last.is_numeric() {
        value.parse().ok()
    } else {
        let val = &value[..value.len() - 1];
        let val: f64 = val.parse().ok()?;
        let mul = match last {
            'K' => 1_000.0,
            'M' => 1_000_000.0,
            modifier => unimplemented!("Unknown modifier '{}'", modifier),
        };

        Some((val * mul) as u64)
    }
}
