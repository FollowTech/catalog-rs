pub trait ToRegValue {
    fn to_reg_value(&self) -> RegValue;
}

impl<$($l,)*> ToRegValue for Vec<$t> {
    fn to_reg_value(&self) -> RegValue {
        let mut os_strings = self
            .into_iter()
            .map(to_utf16)
            .collect::<Vec<_>>()
            .concat();
        os_strings.push(0);
        RegValue {
            bytes: v16_to_v8(&os_strings),
            vtype: REG_MULTI_SZ,
        }
    }
}