/** @type {import('next').NextConfig} */

const nextConfig = {
    reactStrictMode: true,
    swcMinify: true,
    images: {
        unoptimized: true
    },
    webpack: (config) => {
        config.experiments = {
            topLevelAwait: true
        }
        return config
    }
};

module.exports = nextConfig;
