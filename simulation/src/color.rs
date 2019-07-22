use na::Vector3;

#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct Color {
    components: Vector3<u8>,
}

impl Color {
    pub fn new(red: u8, green: u8, blue: u8) -> Color {
        Color {
            components: Vector3::new(red, green, blue),
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = u8> + '_ {
        self.components.iter().cloned()
    }
}
