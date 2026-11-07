fn main() {
    let cases = [
        ("heading", "# Plan\n"),
        ("bold", "Para **medir** el spike\n"),
        ("list", "- uno\n- dos\n"),
        ("link", "[links](https://example.com)\n"),
        ("code block", "```rust\nfn main() {}\n```\n"),
        ("quote", "> cita\n"),
        ("em", "*enfasis*\n"),
        ("inline code", "`x`\n"),
        ("two paragraphs", "a\n\nb\n"),
        ("ordered", "1. a\n2. b\n"),
    ];
    for (name, md) in cases {
        match slint::StyledText::from_markdown(md) {
            Ok(_) => println!("{name}: ok"),
            Err(e) => println!("{name}: ERR {e}"),
        }
    }
}
