const fs=require('fs');
// FolderChip.test.ts: vsep back
let p='src/composer/FolderChip.test.ts';let s=fs.readFileSync(p,'utf8');
s=s.split(`['folder']`).join(`['vsep', 'folder']`);
fs.writeFileSync(p,s);
// Composer.dock.test.ts: vsep counts back to 1, border pin -> no border + vsep rule
p='src/composer/Composer.dock.test.ts';s=fs.readFileSync(p,'utf8');
s=s.split(`toHaveLength(0)`).join(`toHaveLength(1)`);
s=s.replace(`border: 1px solid var(--border)`, `background: var(--surface2)`);
fs.writeFileSync(p,s);console.log('ok');
