use crate::map::Map;

mod map;

fn main() {
    // let mut map = Map::new(510, (10, 10));
    let mut map = Map::new(6084, (14, 709));
    let risk_level = map.get_risk_level((0, 0), (14, 709));

    println!("Risk level: {}", risk_level);

    let distance = map.find_distance_to_target();
    println!("Distance to target: {}", distance);
}
