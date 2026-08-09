const fs = require('fs');
const path = require('path');

const rootDir = path.resolve(__dirname, '..');
const sourceExe = path.join(rootDir, 'src-tauri', 'target', 'release', 'data-tracker.exe');
const sourceInstaller = path.join(rootDir, 'src-tauri', 'target', 'release', 'bundle', 'nsis');
const targetDir = path.join(rootDir, 'releases');

if (!fs.existsSync(targetDir)) {
  fs.mkdirSync(targetDir, { recursive: true });
}

let copied = 0;

if (fs.existsSync(sourceExe)) {
  const dest = path.join(targetDir, 'data-tracker.exe');
  fs.copyFileSync(sourceExe, dest);
  console.log(`Copied: ${sourceExe} -> ${dest}`);
  copied++;
}

if (fs.existsSync(sourceInstaller)) {
  const files = fs.readdirSync(sourceInstaller).filter(f => f.endsWith('.exe') || f.endsWith('.msi'));
  for (const file of files) {
    const src = path.join(sourceInstaller, file);
    const dest = path.join(targetDir, file);
    fs.copyFileSync(src, dest);
    console.log(`Copied: ${src} -> ${dest}`);
    copied++;
  }
}

if (copied === 0) {
  console.warn('No build artifacts found at:', sourceExe, sourceInstaller);
} else {
  console.log(`Done. Copied ${copied} file(s) to releases/`);
}
