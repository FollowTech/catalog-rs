trait Foo {
    fn method<T>(&self, t: T);
}

struct Bar;

impl Bar {
    fn new() -> Self {
        Self {}
    }
}

impl Foo for Bar {
    fn method<T>(&self, t: T) {
        println!("Bar impl trait Foo!")
    }
}

struct InputData {
    payload: Option<Vec<u8>>,
}

fn encrypt(data: &[u8]) -> Vec<u8> {
    data.to_vec()
}

impl InputData {
    pub fn encrypted(&self) -> Vec<u8> {
        encrypt(&self.payload.as_ref().unwrap_or(&vec![]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn as_trait_bound() {
        let bar = Bar::new();
        bar.method(1);
    }

    #[test]
    fn as_impl_trait() {
        let bar = Bar::new();
        let mut v: Vec<Bar> = vec![];
        v.push(bar);
    }

    #[test]
    fn test_as() {
        let x: u32 = 2;
        let y: u64 = x.into();
    }
}
