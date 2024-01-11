import { Suspense } from 'react';

export default async function MetricsLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <Suspense fallback={<div>Loading...</div>}>
      <div className="flex h-full w-full flex-row">{children}</div>
    </Suspense>
  );
}
