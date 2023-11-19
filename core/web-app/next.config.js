/** @type {import('next').NextConfig} */
const nextConfig = {
  swcMinify: true,
  output: 'standalone',
  experimental: {
    appDir: true,
    esmExternals: 'loose',
  },
  /* async redirects() {
    return [
      {
        source: '/app/planning/:id',
        destination: '/app/planning/:id/diagram',
        permanent: true,
      },
    ];
  }, */
  headers:
    process.env.NODE_ENV === 'development'
      ? () => [
          {
            source: '/_next/static/css/_app-client_src_app_globals_css.css',
            headers: [{ key: 'Vary', value: '*' }],
          },
        ]
      : undefined,
};

module.exports = nextConfig;
