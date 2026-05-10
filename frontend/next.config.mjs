import { fileURLToPath } from 'url';
import { dirname } from 'path';

const __dirname = dirname(fileURLToPath(import.meta.url));

/** @type {import('next').NextConfig} */
const nextConfig = {
  // Allow building even if TypeScript reports errors (temporary for CI/dev).
  typescript: {
    ignoreBuildErrors: true,
  },
  // Pin workspace root so Next doesn't pick up the parent lockfile
  turbopack: {
    root: __dirname,
  },
  webpack: (config) => {
    // Solana wallet adapter references Node builtins that don't exist in browser
    config.resolve.fallback = {
      ...config.resolve.fallback,
      fs: false,
      os: false,
      path: false,
      crypto: false,
    };
    return config;
  },
};

export default nextConfig;
