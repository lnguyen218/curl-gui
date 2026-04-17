import sharp from 'sharp';
import fs from 'fs';
import path from 'path';

const svgPath = 'src-tauri/icons/app-icon.svg';
const outputDir = 'src-tauri/icons';

const sizes = [
  { name: '32x32.png', size: 32 },
  { name: '128x128.png', size: 128 },
  { name: '128x128@2x.png', size: 256 },
  { name: 'icon.png', size: 1024 }
];

async function generateIcons() {
  const svgBuffer = fs.readFileSync(svgPath);

  for (const { name, size } of sizes) {
    const outputPath = path.join(outputDir, name);
    await sharp(svgBuffer)
      .resize(size, size)
      .png()
      .toFile(outputPath);
    console.log(`Generated ${name} (${size}x${size})`);
  }

  // For icon.ico, we need multiple sizes - let's use 256 for now
  const icoSizes = [16, 32, 48, 256];
  const icoBuffers = await Promise.all(
    icoSizes.map(s => sharp(svgBuffer).resize(s, s).png().toBuffer())
  );
  
  // Sharp doesn't support ICO directly, so we'll just use the 256 PNG as icon.ico
  // Tauri will handle the conversion during build
  fs.copyFileSync(path.join(outputDir, 'icon.png'), path.join(outputDir, 'icon.ico'));
  console.log('Generated icon.ico (copied from icon.png)');
  
  console.log('\nAll icons generated!');
}

generateIcons().catch(err => {
  console.error('Error:', err);
  process.exit(1);
});
