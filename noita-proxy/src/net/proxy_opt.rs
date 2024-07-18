use std::io::Write;

/// Trait that allows to pass value to mod as proxy options.
pub(crate) trait ProxyOpt {
    fn write_opt(self, buf: &mut Vec<u8>, key: &str);
}

impl ProxyOpt for bool {
    fn write_opt(self, buf: &mut Vec<u8>, key: &str) {
        write!(
            buf,
            "proxy_opt_bool {} {}",
            key,
            if self { "true" } else { "false" }
        )
        .unwrap();
    }
}

impl ProxyOpt for u32 {
    fn write_opt(self, buf: &mut Vec<u8>, key: &str) {
        write!(buf, "proxy_opt_num {} {}", key, self).unwrap();
    }
}

impl ProxyOpt for f32 {
    fn write_opt(self, buf: &mut Vec<u8>, key: &str) {
        write!(buf, "proxy_opt_num {} {}", key, self).unwrap();
    }
}

impl ProxyOpt for &str {
    fn write_opt(self, buf: &mut Vec<u8>, key: &str) {
        write!(buf, "proxy_opt {} {}", key, self).unwrap();
    }
}
