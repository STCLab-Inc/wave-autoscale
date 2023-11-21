export default async function ScalingComponentsLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return <div className="flex h-full w-full flex-row">{children}</div>;
}
