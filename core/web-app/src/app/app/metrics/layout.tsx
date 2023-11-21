export default async function MetricsLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return <div className="flex h-full w-full flex-row">{children}</div>;
}
