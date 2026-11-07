// The markdown of one message of the `flood` scenario: a heading, a paragraph
// with inline formatting, a list and a code block, so a long thread stresses
// the same widgets a real conversation does.

export function floodMessage(index) {
  return [
    `## flood message ${index}`,
    '',
    `This is **message ${index}** of a long thread, with \`inline code\`, *emphasis* and a [link](https://example.com).`,
    '',
    '- first point of the list',
    '- second point of the list',
    '- third point of the list',
    '',
    '```rust',
    `fn message_${index}() {`,
    `    println!("flood {index}");`,
    '}',
    '```',
    '',
  ].join('\n');
}
