use na::Vector3;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Color {
    components: Vector3<u8>,
}

impl Color {
    pub fn from_hex(hex: &str) -> Color {
        fn component(slice: &str) -> u8 {
            u8::from_str_radix(slice, 16).unwrap()
        }

        Color {
            components: Vector3::new(component(&hex[1 .. 3]), component(&hex[3 .. 5]), component(&hex[5 .. 7])),
        }
    }

    pub fn iter(&self) -> impl Iterator<Item=u8> + '_ {
        self.components.iter().cloned()
    }
}
