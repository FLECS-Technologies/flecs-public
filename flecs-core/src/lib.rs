mod cellar;
mod flecs_rest;
pub mod fsm;
pub mod lore;
mod relic;
mod sorcerer;
mod spell;
mod vault;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
