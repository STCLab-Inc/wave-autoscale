export default async function AutoscalingHistoryLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return <div className="flex h-full w-full flex-row">{children}</div>;
}
