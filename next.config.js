/** @type {import('next').NextConfig} */

const nextConfig = {
    reactStrictMode: true,
    swcMinify: true,
    images: {
        unoptimized: true
    },
    webpack: (config) => {
        config.experiments = {
            topLevelAwait: true,
            layers: true
        }
        return config
    },
    experimental: {
        appDir: true,
    },
};

module.exports = nextConfig;
