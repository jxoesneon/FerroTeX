const { chromium } = require('/Users/meilynlopezcubero/.npm/_npx/9833c18b2d85bc59/node_modules/playwright');
const { execSync } = require('child_process');
const path = require('path');
const fs = require('fs');

const VIDEO_DIR = path.resolve('/Users/mey/FerroTex/scripts/videos');
const OUTPUT_DIR = path.resolve('/Users/mey/FerroTex/editors/vscode/images');
const PROFILE_DIR = '/tmp/vscode_showcase_profile';

fs.mkdirSync(VIDEO_DIR, { recursive: true });
fs.mkdirSync(OUTPUT_DIR, { recursive: true });

function convertWebmToGif(webmPath, gifName, startSec, durationSec) {
  const gifPath = path.join(OUTPUT_DIR, gifName);
  console.log(`Converting ${webmPath} (start: ${startSec.toFixed(2)}s, dur: ${durationSec.toFixed(2)}s) -> ${gifPath}...`);
  
  const cmd = `/opt/homebrew/bin/ffmpeg -y -ss ${startSec.toFixed(2)} -t ${durationSec.toFixed(2)} -i "${webmPath}" -vf "fps=15,scale=960:-1:flags=lanczos,split[s0][s1];[s0]palettegen=max_colors=128:stats_mode=diff[p];[s1][p]paletteuse=dither=bayer:bayer_scale=3" -loop 0 "${gifPath}"`;
  execSync(cmd, { stdio: 'inherit' });
  const stat = fs.statSync(gifPath);
  console.log(`Generated ${gifName}: ${(stat.size / 1024).toFixed(1)} KB`);
}

async function createPersistentContext() {
  const context = await chromium.launchPersistentContext(PROFILE_DIR, {
    executablePath: '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome',
    args: ['--lang=en-US', '--disable-features=Translate', '--force-dark-mode'],
    viewport: { width: 1280, height: 720 },
    locale: 'en-US',
    colorScheme: 'dark',
    headless: true,
    recordVideo: { dir: VIDEO_DIR, size: { width: 1280, height: 720 } },
  });

  // Inject persistent CSS to suppress all setup/chat/banner/toast noise
  await context.addInitScript(() => {
    const injectStyle = () => {
      if (!document.getElementById('ciel-clean-style')) {
        const style = document.createElement('style');
        style.id = 'ciel-clean-style';
        style.innerHTML = `
          .part.auxiliarybar,
          .monaco-workbench .part.sidebar.right,
          .part.banner,
          .notifications-toasts,
          .notification-toast,
          .monaco-dialog-modal-block,
          .monaco-modal-editor-block,
          .part.panel {
            display: none !important;
          }
        `;
        document.head.appendChild(style);
      }
    };
    window.addEventListener('DOMContentLoaded', injectStyle);
    setInterval(injectStyle, 200);
  });

  return context;
}

// -------------------------------------------------------------
// Recording 1: Semantic Diagnostics
// -------------------------------------------------------------
async function recordDiagnostics() {
  console.log('\n--- Recording Feature 1: Semantic Diagnostics ---');
  const context = await createPersistentContext();
  const page = context.pages().length > 0 ? context.pages()[0] : await context.newPage();
  
  const pageLoadStart = Date.now();
  await page.goto('http://127.0.0.1:8000/?vscode-locale=en', { waitUntil: 'networkidle' });
  await page.waitForTimeout(2000);

  // Close other editors & bottom panel
  await page.evaluate(() => {
    document.querySelectorAll('.tab .codicon-close').forEach(e => e.click());
    const panelClose = document.querySelector('.part.panel .codicon-panel-close, .part.panel .codicon-close');
    if (panelClose) panelClose.click();
  });

  // Open diagnostics.tex
  const file = page.locator('.monaco-list-row:has-text("diagnostics.tex")');
  if (await file.count() > 0) await file.first().click();
  
  // Wait for squiggles to appear (3-4 seconds)
  await page.waitForTimeout(3500);

  // START RECORDING ACTION HERE
  const actionStart = Date.now();

  // 1. Hover on unmatched \\left line
  const leftLine = page.locator('.view-line:has-text("f(x) = \\\\left")').first();
  if (await leftLine.count() > 0) {
    const box = await leftLine.boundingBox();
    if (box) {
      await page.mouse.move(box.x + 95, box.y + 10, { steps: 20 });
      await page.waitForTimeout(3000); // Allow tooltip to be read
    }
  }

  // 2. Hover on jagged matrix row
  const matrixLine = page.locator('.view-line:has-text("0 & 1")').first();
  if (await matrixLine.count() > 0) {
    const box = await matrixLine.boundingBox();
    if (box) {
      await page.mouse.move(box.x + 80, box.y + 10, { steps: 20 });
      await page.waitForTimeout(3000); // Allow jagged matrix tooltip to be read
    }
  }

  const actionEnd = Date.now();
  const video = page.video();
  await context.close();

  const videoPath = await video.path();
  const startSec = (actionStart - pageLoadStart) / 1000;
  const durationSec = (actionEnd - actionStart) / 1000;
  convertWebmToGif(videoPath, 'feature_diagnostics.gif', startSec, durationSec);
}

// -------------------------------------------------------------
// Recording 2: Rich Documentation Hover
// -------------------------------------------------------------
async function recordHover() {
  console.log('\n--- Recording Feature 2: Rich Hover & Macro Documentation ---');
  const context = await createPersistentContext();
  const page = context.pages().length > 0 ? context.pages()[0] : await context.newPage();

  const pageLoadStart = Date.now();
  await page.goto('http://127.0.0.1:8000/?vscode-locale=en', { waitUntil: 'networkidle' });
  await page.waitForTimeout(2000);

  // Close tabs & panel
  await page.evaluate(() => {
    document.querySelectorAll('.tab .codicon-close').forEach(e => e.click());
    const panelClose = document.querySelector('.part.panel .codicon-panel-close, .part.panel .codicon-close');
    if (panelClose) panelClose.click();
  });

  // Open hover.tex
  const file = page.locator('.monaco-list-row:has-text("hover.tex")');
  if (await file.count() > 0) await file.first().click();
  await page.waitForTimeout(2500);

  // START RECORDING ACTION HERE
  const actionStart = Date.now();

  // 1. Hover on \\usepackage{microtype}
  const microtypeLine = page.locator('.view-line:has-text("usepackage{microtype}")').first();
  if (await microtypeLine.count() > 0) {
    const box = await microtypeLine.boundingBox();
    if (box) {
      await page.mouse.move(box.x + 120, box.y + 10, { steps: 20 });
      await page.waitForTimeout(3000); // Read package doc
    }
  }

  // 2. Hover on \\textbf
  const textbfLine = page.locator('.view-line:has-text("textbf")').first();
  if (await textbfLine.count() > 0) {
    const box = await textbfLine.boundingBox();
    if (box) {
      await page.mouse.move(box.x + 90, box.y + 10, { steps: 20 });
      await page.waitForTimeout(3000); // Read command doc
    }
  }

  const actionEnd = Date.now();
  const video = page.video();
  await context.close();

  const videoPath = await video.path();
  const startSec = (actionStart - pageLoadStart) / 1000;
  const durationSec = (actionEnd - actionStart) / 1000;
  convertWebmToGif(videoPath, 'feature_hover.gif', startSec, durationSec);
}

// -------------------------------------------------------------
// Recording 3: Autocompletion & Snippets
// -------------------------------------------------------------
async function recordCompletion() {
  console.log('\n--- Recording Feature 3: Autocompletion & Snippets ---');
  // Reset completion.tex file cleanly
  fs.writeFileSync('/Users/mey/FerroTex/tests/tex/smoke/completion.tex', `\\documentclass{article}
\\usepackage{amsmath}

\\begin{document}
\\section{Instant Snippets & Autocompletion}



\\end{document}
`);

  const context = await createPersistentContext();
  const page = context.pages().length > 0 ? context.pages()[0] : await context.newPage();

  const pageLoadStart = Date.now();
  await page.goto('http://127.0.0.1:8000/?vscode-locale=en', { waitUntil: 'networkidle' });
  await page.waitForTimeout(2000);

  // Close tabs & panel
  await page.evaluate(() => {
    document.querySelectorAll('.tab .codicon-close').forEach(e => e.click());
    const panelClose = document.querySelector('.part.panel .codicon-panel-close, .part.panel .codicon-close');
    if (panelClose) panelClose.click();
  });

  // Open completion.tex
  const file = page.locator('.monaco-list-row:has-text("completion.tex")');
  if (await file.count() > 0) await file.first().click();
  await page.waitForTimeout(2500);

  // Position cursor on empty line 7
  const line7 = page.locator('.view-line').nth(6);
  await line7.click({ position: { x: 50, y: 10 } });
  await page.waitForTimeout(500);

  // START RECORDING ACTION HERE
  const actionStart = Date.now();

  // Type environment with intelligent autocompletion
  console.log('Typing \\begin{pm...');
  await page.keyboard.type('\\begin{pm', { delay: 60 });
  await page.waitForTimeout(300);

  // Trigger suggest popup
  await page.keyboard.press('Control+Space');
  await page.waitForTimeout(1400); // Show pmatrix suggestion from amsmath

  // Expand suggestion
  await page.keyboard.press('Enter');
  await page.waitForTimeout(400);

  // Insert matrix elements
  await page.keyboard.press('Enter');
  await page.waitForTimeout(300);
  await page.keyboard.type('  1 & 0 \\\\', { delay: 45 });
  await page.waitForTimeout(300);
  await page.keyboard.press('Enter');
  await page.waitForTimeout(300);
  await page.keyboard.type('  0 & 1', { delay: 45 });
  await page.waitForTimeout(300);
  await page.keyboard.press('Enter');
  await page.waitForTimeout(300);
  await page.keyboard.type('\\end{pmatrix}', { delay: 45 });
  await page.waitForTimeout(3000); // Admire completed matrix

  const actionEnd = Date.now();
  const video = page.video();
  await context.close();

  const videoPath = await video.path();
  const startSec = (actionStart - pageLoadStart) / 1000;
  const durationSec = (actionEnd - actionStart) / 1000;
  convertWebmToGif(videoPath, 'feature_completion.gif', startSec, durationSec);
}

// -------------------------------------------------------------
// Recording 4: Zero-Config Build & PDF Preview
// -------------------------------------------------------------
async function recordBuildAndPreview() {
  console.log('\n--- Recording Feature 4: Build & PDF Preview ---');
  // Ensure fresh build exists
  execSync('/Users/mey/FerroTex/editors/vscode/bin/tectonic -o /Users/mey/FerroTex/tests/tex/smoke/build /Users/mey/FerroTex/tests/tex/smoke/build_preview.tex');

  const context = await createPersistentContext();
  const page = context.pages().length > 0 ? context.pages()[0] : await context.newPage();

  const pageLoadStart = Date.now();
  await page.goto('http://127.0.0.1:8000/?vscode-locale=en', { waitUntil: 'networkidle' });
  await page.waitForTimeout(2000);

  // Close tabs & panel
  await page.evaluate(() => {
    document.querySelectorAll('.tab .codicon-close').forEach(e => e.click());
    const panelClose = document.querySelector('.part.panel .codicon-panel-close, .part.panel .codicon-close');
    if (panelClose) panelClose.click();
  });

  // Open build_preview.tex
  const file = page.locator('.monaco-list-row:has-text("build_preview.tex")');
  if (await file.count() > 0) await file.first().click();
  await page.waitForTimeout(2000);

  // START RECORDING ACTION HERE
  const actionStart = Date.now();

  // 1. Trigger Build Document cleanly
  console.log('Triggering build...');
  await page.keyboard.press('F1');
  await page.waitForTimeout(300);
  await page.keyboard.type('FerroTeX: Build Document', { delay: 20 });
  await page.waitForTimeout(300);
  await page.keyboard.press('Enter');
  await page.waitForTimeout(1800);

  // 2. Open PDF Preview beside
  console.log('Triggering PDF Preview...');
  const previewIcon = page.locator('.action-item [aria-label*="FerroTeX: Open PDF Preview"], .action-item .codicon-open-preview');
  if (await previewIcon.count() > 0) {
    await previewIcon.first().click();
  } else {
    await page.keyboard.press('F1');
    await page.waitForTimeout(300);
    await page.keyboard.type('FerroTeX: Open PDF Preview', { delay: 20 });
    await page.waitForTimeout(300);
    await page.keyboard.press('Enter');
  }
  await page.waitForTimeout(5000); // Allow split view and PDF.js to render canvas

  const actionEnd = Date.now();
  const video = page.video();
  await context.close();

  const videoPath = await video.path();
  const startSec = (actionStart - pageLoadStart) / 1000;
  const durationSec = (actionEnd - actionStart) / 1000;
  convertWebmToGif(videoPath, 'feature_preview.gif', startSec, durationSec);
}

(async () => {
  try {
    const feature = process.argv[2];
    if (feature === 'preview') {
      await recordBuildAndPreview();
    } else {
      await recordDiagnostics();
      await recordHover();
      await recordCompletion();
      await recordBuildAndPreview();
    }
    console.log('\nSHOWCASE GIF RECORDED & GENERATED IN ENGLISH DARK MODE CLEANLY!');
  } catch (err) {
    console.error('Recording error:', err);
  }
})();
