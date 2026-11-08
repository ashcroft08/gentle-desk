use super::*;

#[test]
fn truncate_keeps_short_text_and_cuts_long_text_on_a_character_boundary() {
    assert_eq!(truncate("hola", 4), "hola");
    assert_eq!(truncate("hola mundo", 4), "hola…");
    let long = "ñ".repeat(10);
    assert_eq!(truncate(&long, 3), "ñññ…");
}
