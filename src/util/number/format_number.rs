pub trait FormatNumber {
    fn format_big_number(&self) -> String;
}

impl FormatNumber for i64 {
    fn format_big_number(&self) -> String {
        if *self < 0 {
            format!("-{}", (self.unsigned_abs()).format_big_number())
        } else {
            (self.unsigned_abs()).format_big_number()
        }
    }
}

impl FormatNumber for u64 {
    fn format_big_number(&self) -> String {
        if *self < 1000 {
            return self.to_string();
        }

        let l = self.ilog10() / 3;

        let big = 1000_u64.pow(u32::max(0, l));
        let round_denom = 1000_u64.pow(u32::max(0, l - 1));

        let big_part = self / big;
        let small_part = (self % big) / round_denom;

        let leading_zeroes = if small_part > 0 {
            let n_leading_zeroes = 2 - small_part.ilog10();
            (0..n_leading_zeroes).map(|_| "0").collect::<String>()
        } else {
            "".to_string()
        };

        match l {
            1 => format!("{}.{}{}K", big_part, leading_zeroes, small_part),
            2 => format!("{}.{}{}M", big_part, leading_zeroes, small_part),
            3 => format!("{}.{}{}B", big_part, leading_zeroes, small_part),
            4 => format!("{}.{}{}T", big_part, leading_zeroes, small_part),
            _ => self.to_string(),
        }
    }
}
