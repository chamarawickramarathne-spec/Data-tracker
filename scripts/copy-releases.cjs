const fs = require('fs');
const path = require('path');

const rootDir = path.resolve(__dirname, '..');
const targetDir = path.join(rootDir, 'releases');

// x64 builds land in target/release; x86 builds use the i686 target directory.
const targets = {
  x64: path.join(rootDir, 'src-tauri', 'target'),
  x86: path.join(rootDir, 'src-tauri', 'target', 'i686-pc-windows-msvc'),
};

if (!fs.existsSync(targetDir)) {
  fs.mkdirSync(targetDir, { recursive: true });
}

let copied = 0;

function copyFile(src, dest) {
  fs.copyFileSync(src, dest);
  console.log(`Copied: ${src} -> ${dest}`);
  copied++;
}

for (const [arch, base] of Object.entries(targets)) {
  const exe = path.join(base, 'release', 'data-tracker.exe');
  if (fs.existsSync(exe)) {
    copyFile(exe, path.join(targetDir, `data-tracker-${arch}.exe`));
    // Legacy alias expected by older clients (x64 build).
    if (arch === 'x64') {
      copyFile(exe, path.join(targetDir, 'data-tracker.exe'));
    }
  } else {
    console.warn(`[${arch}] exe not found: ${exe}`);
  }

  const installerDir = path.join(base, 'release', 'bundle', 'nsis');
  if (fs.existsSync(installerDir)) {
    const installers = fs
      .readdirSync(installerDir)
      .filter((f) => f.endsWith('.exe') || f.endsWith('.msi'));
    for (const file of installers) {
      copyFile(path.join(installerDir, file), path.join(targetDir, file));
    }
  } else {
    console.warn(`[${arch}] installer dir not found: ${installerDir}`);
  }
}

if (copied === 0) {
  console.warn('No build artifacts found for either architecture.');
} else {
  const byVersion = (a, b) => a.localeCompare(b, undefined, { numeric: true });

  // Keep only the newest versioned installer for each architecture.
  for (const arch of Object.keys(targets)) {
    const re = new RegExp(`^DataTracker_.+_${arch}-setup\\.exe$`);
    const files = fs.readdirSync(targetDir).filter((f) => re.test(f));
    if (files.length === 0) continue;
    const newest = files.sort(byVersion).pop();
    for (const file of files) {
      if (file !== newest) {
        fs.unlinkSync(path.join(targetDir, file));
        console.log(`Removed old installer: ${file}`);
      }
    }
  }

  // The stable site asset mirrors the newest x64 installer.
  const x64Re = /^DataTracker_.+_x64-setup\.exe$/;
  const newestX64 = fs
    .readdirSync(targetDir)
    .filter((f) => x64Re.test(f))
    .sort(byVersion)
    .pop();
  if (newestX64) {
    fs.copyFileSync(path.join(targetDir, newestX64), path.join(targetDir, 'DataTrackerSetup.exe'));
    console.log('Site asset refreshed: DataTrackerSetup.exe');
  }

  console.log(`Done. Copied ${copied} file(s) to releases/`);
}
