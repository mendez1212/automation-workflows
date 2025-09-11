import { Webhooks } from "@octokit/webhooks";
import { Octokit } from "@octokit/rest";
import { createAppAuth } from "@octokit/auth-app";
import sharp from "sharp";
import getRawBody from "raw-body";

// Config: easy to adjust target width & radius %
const config = {
  targetWidth: 300,            // Target width in pixels
  radiusPercentage: 0.065,     // Radius = 6.5% of width
  maxConcurrency: 2,           // Default concurrency for local/dev; overridden in serverless or production.
  maxCacheSize: 1000,          // Maximum number of processed files to cache
  maxSvgCacheSize: 50,         // Maximum number of SVG masks to cache
};

const isServerless = !!process.env.VERCEL || !!process.env.AWS_LAMBDA_FUNCTION_NAME;
config.maxConcurrency = isServerless ? 2 : require("os").cpus().length;

// LRU Cache implementation for processed files
class LRUCache {
  constructor(maxSize) {
    this.maxSize = maxSize;
    this.cache = new Map();
  }

  get(key) {
    if (this.cache.has(key)) {
      // Move to end (most recently used)
      const value = this.cache.get(key);
      this.cache.delete(key);
      this.cache.set(key, value);
      return value;
    }
    return undefined;
  }

  set(key, value) {
    if (this.cache.has(key)) {
      // Update existing
      this.cache.delete(key);
    } else if (this.cache.size >= this.maxSize) {
      // Remove least recently used (first item)
      const firstKey = this.cache.keys().next().value;
      this.cache.delete(firstKey);
    }
    this.cache.set(key, value);
  }

  has(key) {
    return this.cache.has(key);
  }

  size() {
    return this.cache.size;
  }
}

// LRU caches for better memory management
const processedCache = new LRUCache(config.maxCacheSize);
const svgMaskCache = new LRUCache(config.maxSvgCacheSize);

// Enhanced concurrency limiter with better error handling
async function processWithConcurrencyLimit(tasks, limit = config.maxConcurrency) {
  const results = [];
  let completed = 0;
  
  console.log(`Processing ${tasks.length} tasks with concurrency limit of ${limit}`);
  
  for (let i = 0; i < tasks.length; i += limit) {
    const batch = tasks.slice(i, i + limit);
    const batchResults = await Promise.allSettled(batch.map(task => task()));
    
    // Log failed tasks for debugging
    batchResults.forEach((result, index) => {
      if (result.status === 'rejected') {
        console.error(`Task ${i + index + 1} failed:`, result.reason);
      }
    });
    
    results.push(...batchResults);
    completed += batch.length;
    console.log(`Completed ${completed}/${tasks.length} tasks`);
  }
  
  const successful = results.filter(r => r.status === 'fulfilled').length;
  const failed = results.filter(r => r.status === 'rejected').length;
  console.log(`Batch processing complete: ${successful} successful, ${failed} failed`);
  
  return results;
}

async function checkIfImageNeedsProcessing(inputBuffer, filePath) {
  try {
    // Early exit: Check if it's PNG by magic bytes (more efficient than sharp metadata)
    const pngSignature = Buffer.from([0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);
    if (!inputBuffer.subarray(0, 8).equals(pngSignature)) {
      console.log(`Early exit: ${filePath} is not a PNG file (magic bytes check)`);
      return { needsProcessing: false, reason: "Not a PNG file", resizedBuffer: inputBuffer };
    }

    const metadata = await sharp(inputBuffer).metadata();

    // Double-check format (in case magic bytes were spoofed)
    if (metadata.format !== "png") {
      console.log(`Early exit: ${filePath} format is ${metadata.format}, not PNG`);
      return { needsProcessing: false, reason: "Not a PNG file", resizedBuffer: inputBuffer };
    }

    console.log(`Image ${filePath} dimensions: ${metadata.width}x${metadata.height}`);

    if (metadata.width !== config.targetWidth) {
      // Resize once here and reuse the sharp instance for metadata
      const sharpInstance = sharp(inputBuffer).resize({ width: config.targetWidth });
      const resizedBuffer = await sharpInstance.toBuffer();
      
      // Get metadata from the same sharp instance (more efficient)
      const resizedSharp = sharp(resizedBuffer);
      const resizedMetadata = await resizedSharp.metadata();

      const hasRoundedCorners = await checkForRoundedCorners(resizedBuffer, resizedMetadata);

      if (!hasRoundedCorners) {
        console.log(`${filePath}: Width resized (${metadata.width}px → ${config.targetWidth}px) and corners need rounding`);
        return {
          needsProcessing: true,
          reason: `Width resized from ${metadata.width}px and corner needs rounding`,
          resizedBuffer,
        };
      }

      console.log(`${filePath}: Width resized (${metadata.width}px → ${config.targetWidth}px) but corners are already rounded`);
      return {
        needsProcessing: true,
        reason: `Width resized from ${metadata.width}px but corner is fine`,
        resizedBuffer,
      };
    }

    // Check corners on original size image
    const hasRoundedCorners = await checkForRoundedCorners(inputBuffer, metadata);

    if (!hasRoundedCorners) {
      console.log(`${filePath}: Correct width (${metadata.width}px) but corners need rounding`);
      return {
        needsProcessing: true,
        reason: "Top-right corner is not rounded as expected",
        resizedBuffer: inputBuffer,
      };
    }

    console.log(`${filePath}: Already optimized (${config.targetWidth}px width with rounded corners)`);
    return {
      needsProcessing: false,
      reason: `Image already meets requirements (${config.targetWidth}px width with rounded top-right corner)`,
      resizedBuffer: inputBuffer,
    };
  } catch (error) {
    console.error(`Error checking image requirements for ${filePath}:`, {
      error: error.message,
      stack: error.stack?.split('\n')[0], // First line of stack trace
      bufferSize: inputBuffer.length
    });
    return { 
      needsProcessing: true, 
      reason: `Error checking image (${error.message}), will process to be safe`, 
      resizedBuffer: inputBuffer 
    };
  }
}

// Enhanced corner detection with better logging
async function checkForRoundedCorners(inputBuffer, metadata) {
  try {
    const radius = Math.round(metadata.width * config.radiusPercentage);
    const cornerSize = Math.min(radius + 5, 20);

    const { data, info } = await sharp(inputBuffer)
      .extract({
        left: metadata.width - cornerSize,
        top: 0,
        width: cornerSize,
        height: cornerSize,
      })
      .raw()
      .toBuffer({ resolveWithObject: true });

    const channels = info.channels;
    const sampleCount = Math.max(3, Math.round(metadata.width / 50)); // Minimum 3 samples

    console.log(`Corner check: radius=${radius}px, samples=${sampleCount}, cornerSize=${cornerSize}px`);

    // Spread sample points evenly from 30° to 80° at fractions 0.5 to 0.9 radius
    const angleStart = 30;
    const angleEnd = 80;
    let transparentPixels = 0;
    
    for (let i = 0; i < sampleCount; i++) {
      const fraction = 0.5 + (i / (sampleCount - 1)) * 0.4; // 0.5 to 0.9 radius fraction
      const angleDeg = angleStart + (i / (sampleCount - 1)) * (angleEnd - angleStart);
      const angleRad = (angleDeg * Math.PI) / 180;

      const dx = Math.round(fraction * radius * Math.cos(angleRad));
      const dy = Math.round(fraction * radius * Math.sin(angleRad));

      const x = cornerSize - 1 - dx;
      const y = dy;

      // Bounds checking
      if (x >= 0 && x < cornerSize && y >= 0 && y < cornerSize) {
        const alphaIndex = (y * cornerSize + x) * channels + 3; // alpha channel
        
        if (data[alphaIndex] < 255) {
          transparentPixels++;
        }
      }
    }

    const hasRoundedCorners = transparentPixels > 0;
    console.log(`Corner detection: ${transparentPixels}/${sampleCount} transparent pixels found, rounded=${hasRoundedCorners}`);
    
    return hasRoundedCorners;
  } catch (error) {
    console.error("Error checking top-right corner for rounded radius:", {
      error: error.message,
      width: metadata.width,
      height: metadata.height
    });
    return false;
  }
}

// Enhanced SVG mask creation with caching
async function processImageBuffer(inputBuffer, filePath) {
  const metadata = await sharp(inputBuffer).metadata();
  const radius = Math.round(metadata.width * config.radiusPercentage);
  const cacheKey = `${metadata.width}-${radius}`;

  let svgMaskBuffer = svgMaskCache.get(cacheKey);
  if (!svgMaskBuffer) {
    console.log(`Creating new SVG mask for ${metadata.width}x${metadata.height} with ${radius}px radius`);
    const svgMask = `
      <svg width="${metadata.width}" height="${metadata.height}">
        <rect x="0" y="0" width="${metadata.width}" height="${metadata.height}" rx="${radius}" ry="${radius}" />
      </svg>
    `;
    svgMaskBuffer = Buffer.from(svgMask);
    svgMaskCache.set(cacheKey, svgMaskBuffer);
    console.log(`SVG mask cached. Cache size: ${svgMaskCache.size()}/${config.maxSvgCacheSize}`);
  } else {
    console.log(`Using cached SVG mask for ${metadata.width}x${metadata.height}`);
  }

  console.log(`Applying rounded corners to ${filePath}`);
  return await sharp(inputBuffer)
    .composite([{ input: svgMaskBuffer, blend: "dest-in" }])
    .png({ quality: 100, compressionLevel: 2 })
    .toBuffer();
}

// Enhanced retry function with exponential backoff
async function retry(fn, retries = 3, delayMs = 1000, operation = "operation") {
  for (let attempt = 1; attempt <= retries; attempt++) {
    try {
      return await fn();
    } catch (e) {
      console.error(`${operation} attempt ${attempt}/${retries} failed:`, e.message);
      if (attempt === retries) {
        console.error(`${operation} failed after ${retries} attempts`);
        throw e;
      }
      
      // Exponential backoff with jitter
      const delay = delayMs * Math.pow(2, attempt - 1) + Math.random() * 1000;
      console.log(`Waiting ${Math.round(delay)}ms before retry...`);
      await new Promise((r) => setTimeout(r, delay));
    }
  }
}

async function processFile(filePath, contentData, octokit, owner, repo, branch, repoFullName) {
  const startTime = Date.now();
  
  try {
    const cacheKey = `${repoFullName}:${branch}:${filePath}`;

    // Check cache first
    if (processedCache.get(cacheKey) === contentData.sha) {
      console.log(`✓ Cache hit: Skipping already processed file: ${repoFullName}/${filePath}`);
      return { status: 'cached', filePath };
    }

    const fileBuffer = Buffer.from(contentData.content, "base64");
    console.log(`Processing ${repoFullName}/${filePath} (${Math.round(fileBuffer.length / 1024)}KB)`);

    const { needsProcessing, reason, resizedBuffer } = await checkIfImageNeedsProcessing(fileBuffer, filePath);

    if (!needsProcessing) {
      console.log(`✓ Skipping ${repoFullName}/${filePath}: ${reason}`);
      processedCache.set(cacheKey, contentData.sha);
      console.log(`Cache updated. Size: ${processedCache.size()}/${config.maxCacheSize}`);
      return { status: 'skipped', filePath, reason };
    }

    console.log(`🔄 Processing ${repoFullName}/${filePath}: ${reason}`);

    const processedBuffer = await processImageBuffer(resizedBuffer, filePath);

    // Compare buffers more efficiently
    if (fileBuffer.length === processedBuffer.length && fileBuffer.equals(processedBuffer)) {
      console.log(`✓ No changes after processing for: ${repoFullName}/${filePath}`);
      processedCache.set(cacheKey, contentData.sha);
      return { status: 'no_changes', filePath };
    }

    console.log(`📤 Committing changes for ${repoFullName}/${filePath}`);
    const { data: commitResult } = await retry(
      () => octokit.rest.repos.createOrUpdateFileContents({
        owner,
        repo,
        path: filePath,
        message: `Auto processed image: rounded corners and resize to ${config.targetWidth}px`,
        content: processedBuffer.toString("base64"),
        sha: contentData.sha,
        branch,
      }),
      3,
      1000,
      `GitHub API commit for ${filePath}`
    );

    const processingTime = Date.now() - startTime;
    console.log(`✅ Successfully processed and committed ${repoFullName}/${filePath} in ${processingTime}ms`);
    
    processedCache.set(cacheKey, commitResult.content.sha);
    console.log(`Cache updated. Size: ${processedCache.size()}/${config.maxCacheSize}`);
    
    return { 
      status: 'processed', 
      filePath, 
      processingTime,
      originalSize: fileBuffer.length,
      newSize: processedBuffer.length
    };
    
  } catch (error) {
    const processingTime = Date.now() - startTime;
    console.error(`❌ Error processing ${repoFullName}/${filePath} after ${processingTime}ms:`, {
      error: error.message,
      stack: error.stack?.split('\n').slice(0, 3).join('\n'), // First 3 lines of stack
      sha: contentData.sha
    });
    return { status: 'error', filePath, error: error.message };
  }
}

// Validate environment variables at startup
function validateEnvironment() {
  const requiredEnvVars = ["GITHUB_WEBHOOK_SECRET", "APP_ID", "PRIVATE_KEY"];
  const missingVars = requiredEnvVars.filter((varName) => !process.env[varName]);
  
  if (missingVars.length > 0) {
    throw new Error(`Missing required environment variables: ${missingVars.join(', ')}`);
  }
  
  console.log("✓ All required environment variables are present");
}

// Initialize webhooks with enhanced error handling
let webhooks;
try {
  validateEnvironment();
  webhooks = new Webhooks({
    secret: process.env.GITHUB_WEBHOOK_SECRET,
  });
  console.log("✓ Webhooks initialized successfully");
} catch (error) {
  console.error("❌ Failed to initialize webhooks:", error.message);
}

if (webhooks) {
  webhooks.on("push", async ({ payload }) => {
    const startTime = Date.now();
    const repoFullName = payload.repository.full_name;
    const [owner, repo] = repoFullName.split("/");
    const branch = payload.ref.replace("refs/heads/", "");

    console.log(`\n🔔 Webhook received: Push to ${repoFullName}/${branch}`);

    // Early exit: non-main branch
    if (branch !== "main") {
      console.log(`⏭️  Skipping push to branch '${branch}', only processing 'main' branch`);
      return;
    }

    // Early exit: no installation ID
    if (!payload.installation?.id) {
      console.log("❌ No installation ID found in payload");
      return;
    }
    const installationId = payload.installation.id;

    // Early exit: bot-generated commit
    if (payload.commits.some((commit) => commit.message.includes("Auto processed image"))) {
      console.log(`🤖 Skipping bot-generated commit in ${repoFullName}`);
      return;
    }

    // Collect all changed files
    const changedFiles = new Set();
    for (const commit of payload.commits) {
      for (const file of [...commit.added, ...commit.modified]) {
        changedFiles.add(file);
      }
    }

    // Early exit: no PNG files in docs/ui/
    const pngFiles = [...changedFiles].filter(
      (file) => file.startsWith("docs/ui/") && file.toLowerCase().endsWith(".png")
    );

    if (pngFiles.length === 0) {
      console.log(`⏭️  No PNG files in docs/ui/ to process in push to ${repoFullName}`);
      return;
    }

    console.log(`📋 Found ${pngFiles.length} PNG files to process:`, pngFiles);

    try {
      const octokit = new Octokit({
        authStrategy: createAppAuth,
        auth: {
          appId: process.env.APP_ID,
          privateKey: process.env.PRIVATE_KEY.replace(/\\n/g, "\n"),
          installationId,
        },
      });

      // Batch fetch file contents
      console.log(`📥 Fetching content for ${pngFiles.length} files...`);
      const contentMap = new Map();
      
      for (const filePath of pngFiles) {
        try {
          const { data } = await retry(
            () => octokit.rest.repos.getContent({
              owner,
              repo,
              path: filePath,
              ref: branch,
            }),
            3,
            1000,
            `fetch content for ${filePath}`
          );
          contentMap.set(filePath, data);
          console.log(`✓ Fetched ${filePath} (${Math.round(Buffer.from(data.content, 'base64').length / 1024)}KB)`);
        } catch (error) {
          console.error(`❌ Failed to fetch content for ${filePath}:`, error.message);
        }
      }

      if (contentMap.size === 0) {
        console.log("❌ No file contents could be fetched");
        return;
      }

      // Create processing tasks
      const tasks = [...contentMap.entries()].map(
        ([filePath, contentData]) =>
          () => processFile(filePath, contentData, octokit, owner, repo, branch, repoFullName)
      );

      // Process with concurrency limit
      console.log(`🚀 Starting batch processing of ${tasks.length} files...`);
      const results = await processWithConcurrencyLimit(tasks, config.maxConcurrency);

      // Summary statistics
      const successful = results.filter(r => r.status === 'fulfilled');
      const failed = results.filter(r => r.status === 'rejected');
      const processed = successful.filter(r => r.value?.status === 'processed');
      const cached = successful.filter(r => r.value?.status === 'cached');
      const skipped = successful.filter(r => r.value?.status === 'skipped');
      const noChanges = successful.filter(r => r.value?.status === 'no_changes');

      const totalTime = Date.now() - startTime;
      
      console.log(`\n📊 Processing Summary for ${repoFullName}:`);
      console.log(`   Total files: ${pngFiles.length}`);
      console.log(`   Processed: ${processed.length}`);
      console.log(`   Cached (skipped): ${cached.length}`);
      console.log(`   Skipped (no processing needed): ${skipped.length}`);
      console.log(`   No changes after processing: ${noChanges.length}`);
      console.log(`   Failed: ${failed.length}`);
      console.log(`   Total time: ${totalTime}ms`);
      console.log(`   Cache sizes: Processed=${processedCache.size()}/${config.maxCacheSize}, SVG=${svgMaskCache.size()}/${config.maxSvgCacheSize}`);

      if (processed.length > 0) {
        const avgTime = processed.reduce((sum, r) => sum + (r.value?.processingTime || 0), 0) / processed.length;
        console.log(`   Average processing time: ${Math.round(avgTime)}ms per file`);
      }

    } catch (error) {
      console.error(`❌ Error processing webhook for ${repoFullName}:`, {
        error: error.message,
        stack: error.stack?.split('\n').slice(0, 3).join('\n')
      });
    }
  });
}

export const apiconfig = {
  api: {
    bodyParser: false,
  },
};

export default async function handler(req, res) {
  if (req.method !== "POST") {
    res.status(405).send("Method Not Allowed");
    return;
  }

  if (!webhooks) {
    console.error("❌ Webhooks not initialized - check GITHUB_WEBHOOK_SECRET environment variable");
    res.status(500).send("Webhook service unavailable");
    return;
  }

  let rawBody;
  try {
    rawBody = await getRawBody(req, { encoding: 'utf8' });
  } catch (err) {
    console.error("❌ Error reading raw body:", err.message);
    res.status(400).send("Invalid raw body");
    return;
  }

  try {
    await webhooks.verifyAndReceive({
      id: req.headers["x-github-delivery"],
      name: req.headers["x-github-event"],
      signature: req.headers["x-hub-signature-256"],
      payload: rawBody,
    });
    res.status(200).send("OK");
  } catch (error) {
    console.error("❌ Webhook verification failed:", error.message);
    res.status(400).send("Invalid signature");
  }
}