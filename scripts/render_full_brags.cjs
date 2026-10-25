const fs = require("fs");
const path = require("path");
const { spawn } = require("child_process");
const {
  chromium,
} = require("/Users/meilynlopezcubero/.npm/_npx/9833c18b2d85bc59/node_modules/playwright");

const MUSIC_PROJECT =
  "/Users/meilynlopezcubero/.gemini/config/skills/brag/assets/music/happy-beats-business-moves-vol-1-by-ende-dot-app.mp3";
const MUSIC_EXTENSION =
  "/Users/meilynlopezcubero/.gemini/config/skills/brag/assets/music/happy-beats-business-moves-vol-11-by-ende-dot-app.mp3";

async function renderBrag(config) {
  console.log(`\n======================================================`);
  console.log(`Starting render for: ${config.name}`);
  console.log(`Composition: ${config.htmlPath}`);
  console.log(`Output Video: ${config.outputMp4}`);
  console.log(`======================================================`);

  const browser = await chromium.launch({
    executablePath: "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
    headless: true,
  });

  const page = await browser.newPage({
    viewport: { width: 1280, height: 720 },
    deviceScaleFactor: 1,
  });

  await page.goto(config.htmlPath, { waitUntil: "load" });
  await page.waitForTimeout(500);

  // 1. Capture Poster Frame (at settled timestamp, e.g. 5.5s)
  console.log(`Capturing settled poster frame at t = ${config.posterTime}s...`);
  await page.evaluate((t) => window.renderAt(t), config.posterTime);
  await page.waitForTimeout(100);
  await page.screenshot({ path: config.posterJpg, type: "jpeg", quality: 95 });
  console.log(`Saved poster image: ${config.posterJpg}`);

  // 2. Setup FFmpeg image2pipe process
  const fps = 30;
  const duration = 20.0;
  const totalFrames = Math.round(duration * fps); // 600 frames
  const tempRawMp4 = config.outputMp4.replace(".mp4", "_raw.mp4");

  console.log(`Spawning FFmpeg pipeline for ${totalFrames} frames @ ${fps} fps...`);
  const ffmpeg = spawn("/opt/homebrew/bin/ffmpeg", [
    "-y",
    "-f",
    "image2pipe",
    "-vcodec",
    "png",
    "-r",
    String(fps),
    "-i",
    "-",
    "-c:v",
    "libx264",
    "-pix_fmt",
    "yuv420p",
    "-preset",
    "fast",
    "-crf",
    "18",
    tempRawMp4,
  ]);

  ffmpeg.stderr.on("data", (d) => {
    // suppress verbose output, only log fatal
    const str = d.toString();
    if (str.includes("Error") || str.includes("fatal")) {
      console.error("FFmpeg stderr:", str);
    }
  });

  let posterBuffer = null;

  // 3. Render all frames sequentially
  for (let i = 0; i < totalFrames; i++) {
    const t = i / fps;
    await page.evaluate((currTime) => window.renderAt(currTime), t);

    // In accordance with creative laws: Frame 0 is baked with settled poster frame
    let frameBuf;
    if (i === 0) {
      await page.evaluate((currTime) => window.renderAt(currTime), config.posterTime);
      frameBuf = await page.screenshot({ type: "png" });
      posterBuffer = frameBuf;
      await page.evaluate((currTime) => window.renderAt(currTime), 0);
    } else {
      frameBuf = await page.screenshot({ type: "png" });
    }

    ffmpeg.stdin.write(frameBuf);

    if (i % 60 === 0 || i === totalFrames - 1) {
      process.stdout.write(
        `Rendering: ${Math.round((i / totalFrames) * 100)}% (${i}/${totalFrames} frames)\r`,
      );
    }
  }

  console.log(`\nAll frames piped. Finalizing raw video encode...`);
  ffmpeg.stdin.end();

  await new Promise((resolve, reject) => {
    ffmpeg.on("close", (code) => {
      if (code === 0) resolve();
      else reject(new Error(`FFmpeg exited with code ${code}`));
    });
  });

  await browser.close();

  // 4. Mux Audio with smart fade-in and fade-out
  console.log(`Muxing soundtrack: ${config.musicTrack}...`);
  await new Promise((resolve, reject) => {
    const mux = spawn("/opt/homebrew/bin/ffmpeg", [
      "-y",
      "-i",
      tempRawMp4,
      "-ss",
      "0",
      "-t",
      String(duration),
      "-i",
      config.musicTrack,
      "-filter_complex",
      "[1:a]afade=t=in:ss=0:d=0.8,afade=t=out:st=18.5:d=1.5,volume=0.85[a]",
      "-map",
      "0:v",
      "-map",
      "[a]",
      "-c:v",
      "copy",
      "-c:a",
      "aac",
      "-b:a",
      "192k",
      "-shortest",
      config.outputMp4,
    ]);

    mux.on("close", (code) => {
      if (code === 0) resolve();
      else reject(new Error(`Audio mux failed with code ${code}`));
    });
  });

  // Clean raw mp4
  if (fs.existsSync(tempRawMp4)) fs.unlinkSync(tempRawMp4);
  console.log(`Rendered high-definition MP4: ${config.outputMp4}`);

  // 5. Generate optimized README GIF (20 fps, 800px width, custom color palette)
  console.log(`Generating optimized showcase GIF: ${config.outputGif}...`);
  await new Promise((resolve, reject) => {
    const gifGen = spawn("/opt/homebrew/bin/ffmpeg", [
      "-y",
      "-i",
      config.outputMp4,
      "-vf",
      "fps=15,scale=800:-1:flags=lanczos,split[s0][s1];[s0]palettegen=max_colors=128[p];[s1][p]paletteuse=dither=bayer:bayer_scale=3",
      config.outputGif,
    ]);

    gifGen.on("close", (code) => {
      if (code === 0) resolve();
      else reject(new Error(`GIF generation failed with code ${code}`));
    });
  });

  console.log(`Generated GIF: ${config.outputGif}`);

  // Copy GIF to extra destination if requested
  if (config.extraGifDest) {
    fs.copyFileSync(config.outputGif, config.extraGifDest);
    console.log(`Copied GIF to: ${config.extraGifDest}`);
  }

  console.log(`Finished ${config.name} successfully!\n`);
}

(async () => {
  try {
    // 1. Full /brag for the Project
    await renderBrag({
      name: "Project Launch Video (GitHub)",
      htmlPath: "file:///Users/mey/FerroTex/brag-project/work/composition.html",
      outputMp4: "/Users/mey/FerroTex/brag-project/brag.mp4",
      outputGif: "/Users/mey/FerroTex/brag-project/brag.gif",
      posterJpg: "/Users/mey/FerroTex/brag-project/brag.jpg",
      extraGifDest: "/Users/mey/FerroTex/editors/vscode/images/brag_project.gif",
      musicTrack: MUSIC_PROJECT,
      posterTime: 5.5,
    });

    // 2. Full /brag for the VS Code Extension
    await renderBrag({
      name: "VS Code Extension Launch Video (Marketplace)",
      htmlPath: "file:///Users/mey/FerroTex/brag-extension/work/composition.html",
      outputMp4: "/Users/mey/FerroTex/brag-extension/brag.mp4",
      outputGif: "/Users/mey/FerroTex/brag-extension/brag.gif",
      posterJpg: "/Users/mey/FerroTex/brag-extension/brag.jpg",
      extraGifDest: "/Users/mey/FerroTex/editors/vscode/images/brag_extension.gif",
      musicTrack: MUSIC_EXTENSION,
      posterTime: 5.5,
    });

    console.log("ALL FULL BRAG LAUNCH DELIVERABLES GENERATED SUCCESSFULLY!");
  } catch (err) {
    console.error("Fatal render error:", err);
    process.exit(1);
  }
})();
