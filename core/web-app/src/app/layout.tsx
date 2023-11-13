import './globals.css';

export const metadata = {
  title: 'Wave Autoscale',
  description:
    'Beyond CPU Utilization Scaling, Handle Traffic Surges and Scale with Confidence',
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en" data-theme="winter">
      <body>{children}</body>
    </html>
  );
}
